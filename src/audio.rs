#[cfg(not(target_arch = "wasm32"))]
mod native {
    use rodio::{Decoder, OutputStream, Sink, Source};
    use std::cell::Cell;
    use std::fs;
    use std::io::BufReader;
    use std::path::Path;

    pub struct AudioManager {
        _stream: Option<OutputStream>,
        stream_handle: Option<rodio::OutputStreamHandle>,
        sink: Option<Sink>,
        dirt_sounds: Vec<Vec<u8>>,
        metal_sounds: Vec<Vec<u8>>,
        step_index: Cell<usize>,
        volume: Cell<f32>,
    }

    impl AudioManager {
        pub fn new() -> Self {
            let (stream, handle, sink) = match OutputStream::try_default() {
                Ok((s, h)) => {
                    let sink = Sink::try_new(&h).ok();
                    if let Some(ref sk) = sink {
                        sk.set_volume(0.5);
                    }
                    (Some(s), Some(h), sink)
                }
                Err(e) => {
                    log::warn!("AudioManager: no audio device ({e}), running muted");
                    (None, None, None)
                }
            };

            let dirt_sounds = Self::load_sounds("assets/sounds", "dirt");
            let metal_sounds = Self::load_sounds("assets/sounds", "metal");

            log::info!(
                "AudioManager: loaded {} dirt, {} metal footstep sounds",
                dirt_sounds.len(),
                metal_sounds.len()
            );

            Self {
                _stream: stream,
                stream_handle: handle,
                sink,
                dirt_sounds,
                metal_sounds,
                step_index: Cell::new(0),
                volume: Cell::new(0.5),
            }
        }

        /// 0-100 slider value.
        pub fn set_volume(&self, v: u32) {
            let f = (v.min(100) as f32) / 100.0;
            self.volume.set(f);
            if let Some(ref sink) = self.sink {
                // Master volume scaled so footsteps stay subtle.
                sink.set_volume(f * 0.8);
            }
        }

        fn load_sounds(dir: &str, prefix: &str) -> Vec<Vec<u8>> {
            let mut sounds = Vec::new();
            let path = Path::new(dir);
            if !path.exists() {
                return sounds;
            }
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if let Some(name) = p.file_stem().and_then(|s| s.to_str()) {
                        if name.starts_with(prefix) && p.extension().and_then(|e| e.to_str()) == Some("wav") {
                            if let Ok(data) = fs::read(&p) {
                                sounds.push(data);
                            }
                        }
                    }
                }
            }
            sounds.sort_by(|a, b| a.len().cmp(&b.len()));
            sounds
        }

        fn append_one_shot(&self, bytes: Vec<u8>, amplify: f32) {
            let (Some(ref handle), Some(ref _sink)) = (&self.stream_handle, &self.sink) else {
                return;
            };
            // Keep the queue short so rapid footsteps don't pile up forever,
            // but don't cut currently playing sounds (old `stop()` behaviour).
            if _sink.len() > 4 {
                return;
            }
            let cursor = std::io::Cursor::new(bytes);
            if let Ok(decoder) = Decoder::new(BufReader::new(cursor)) {
                let source = decoder.amplify(amplify);
                // A dedicated sink per one-shot allows overlaps without
                // stopping the master footstep sink.
                if let Ok(one) = Sink::try_new(handle) {
                    one.set_volume(self.volume.get());
                    one.append(source);
                    one.detach();
                } else {
                    _sink.append(source);
                }
            }
        }

        pub fn play_footstep(&self) {
            if self.dirt_sounds.is_empty() {
                return;
            }
            let idx = self.step_index.get() % self.dirt_sounds.len();
            self.step_index.set(self.step_index.get().wrapping_add(1));
            // Alternate dirt/metal for surface variety.
            let use_metal = !self.metal_sounds.is_empty() && idx % 3 == 2;
            let bytes = if use_metal {
                self.metal_sounds[idx % self.metal_sounds.len()].clone()
            } else {
                self.dirt_sounds[idx].clone()
            };
            self.append_one_shot(bytes, 0.6);
        }

        pub fn play_ui_sound(&self) {
            if self.dirt_sounds.is_empty() {
                return;
            }
            // Play first dirt sound at low volume for UI events, without affecting step index
            self.append_one_shot(self.dirt_sounds[0].clone(), 0.18);
        }

        pub fn play_jump(&self) {
            if self.dirt_sounds.is_empty() {
                return;
            }
            self.append_one_shot(self.dirt_sounds[0].clone(), 0.35);
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::cell::RefCell;

    thread_local! {
        static AUDIO_CONTEXT: RefCell<Option<web_sys::AudioContext>> = RefCell::new(None);
    }

    fn with_ctx<F, R>(f: F) -> Option<R>
    where
        F: FnOnce(&web_sys::AudioContext) -> R,
    {
        AUDIO_CONTEXT.with(|cell| {
            let mut borrow = cell.borrow_mut();
            let ctx = borrow.get_or_insert_with(|| {
                web_sys::AudioContext::new().expect("failed to create AudioContext")
            });
            Some(f(ctx))
        })
    }

    pub struct AudioManager {
        volume: f32,
    }

    impl AudioManager {
        pub fn new() -> Self {
            Self { volume: 0.5 }
        }

        pub fn set_volume(&self, v: u32) {
            let f = (v.min(100) as f32) / 100.0;
            // Volume is stored; individual sources will be set when we have real audio.
            // For now this is a no-op beyond storing the value.
            let _ = f;
        }

        fn play_tone(&self, frequency: f32, duration: f64, gain: f32) {
            let Some(ctx) = with_ctx(|c| c.clone()) else {
                return;
            };

            let Ok(oscillator) = ctx.create_oscillator() else {
                return;
            };
            let Ok(gain_node) = ctx.create_gain() else {
                return;
            };

            oscillator.set_type(web_sys::OscillatorType::Sine);
            oscillator.frequency().set_value(frequency);

            let vol = gain * self.volume;
            let _ = gain_node.gain().set_value_at_time(vol, ctx.current_time());

            let dest = ctx.destination();
            let dest_node: web_sys::AudioNode = dest.into();
            let _ = oscillator.connect_with_audio_node(&gain_node);
            let _ = gain_node.connect_with_audio_node(&dest_node);
            let _ = oscillator.start();
            let _ = oscillator.stop_with_when(ctx.current_time() + duration);
        }

        pub fn play_footstep(&self) {
            self.play_tone(80.0 + (js_sys::Math::random() * 20.0) as f32, 0.06, 0.3);
        }

        pub fn play_ui_sound(&self) {
            self.play_tone(440.0, 0.05, 0.15);
        }

        pub fn play_jump(&self) {
            self.play_tone(220.0, 0.12, 0.25);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::AudioManager;

#[cfg(target_arch = "wasm32")]
pub use web::AudioManager;
