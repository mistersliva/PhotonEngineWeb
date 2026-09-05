#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::path::Path;

    pub struct VideoPlayer {
        input_context: ffmpeg_next::format::context::Input,
        decoder: ffmpeg_next::decoder::Video,
        scaler: ffmpeg_next::software::scaling::Context,
        stream_index: usize,
        time_base: f64,
        duration_secs: f64,
        pixels: Vec<u8>,
        width: u32,
        height: u32,
        pub playing: bool,
        pub loop_video: bool,
        pub alpha: f32,
    }

    impl VideoPlayer {
        pub fn open(path: &str) -> Result<Self, String> {
            let _ = ffmpeg_next::init();

            let path_obj = Path::new(path);
            if !path_obj.exists() {
                return Err(format!("Video file not found: {}", path));
            }

            let context = ffmpeg_next::format::input(&path)
                .map_err(|e| format!("Failed to open video: {}", e))?;

            let stream = context
                .streams()
                .best(ffmpeg_next::media::Type::Video)
                .ok_or("No video stream found")?;

            let stream_index = stream.index();
            let time_base = stream.time_base().0 as f64 / stream.time_base().1 as f64;
            let duration_secs = stream.duration() as f64 * time_base;
            let params = stream.parameters();

            let context_codec = ffmpeg_next::codec::context::Context::from_parameters(params)
                .map_err(|e| format!("Failed to get codec context: {}", e))?;

            let decoder = context_codec
                .decoder()
                .video()
                .map_err(|e| format!("Failed to create video decoder: {}", e))?;

            let width = decoder.width();
            let height = decoder.height();

            let scaler = ffmpeg_next::software::scaling::Context::get(
                decoder.format(),
                width,
                height,
                ffmpeg_next::format::Pixel::RGBA,
                width,
                height,
                ffmpeg_next::software::scaling::Flags::BILINEAR,
            )
            .map_err(|e| format!("Failed to create scaler: {}", e))?;

            let pixels = vec![0u8; (width * height * 4) as usize];

            Ok(Self {
                input_context: context,
                decoder,
                scaler,
                stream_index,
                time_base,
                duration_secs,
                pixels,
                width,
                height,
                playing: true,
                loop_video: true,
                alpha: 1.0,
            })
        }

        pub fn update(&mut self, _dt: f32) -> bool {
            if !self.playing {
                return false;
            }

            // Decode at most a few packets per frame so a slow video can't
            // stall the menu. Returns true when a fresh frame was produced.
            let mut packets_used = 0;
            for (_stream, packet) in self.input_context.packets() {
                if _stream.index() != self.stream_index {
                    continue;
                }
                packets_used += 1;
                if self.decoder.send_packet(&packet).is_err() {
                    if packets_used >= 8 {
                        break;
                    }
                    continue;
                }
                let mut frame = ffmpeg_next::frame::Video::empty();
                while self.decoder.receive_frame(&mut frame).is_ok() {
                    let mut out = ffmpeg_next::frame::Video::empty();
                    self.scaler.run(&frame, &mut out).ok();
                    let data = out.data(0);
                    let copy_len = self.pixels.len().min(data.len());
                    self.pixels[..copy_len].copy_from_slice(&data[..copy_len]);
                    return true;
                }
                if packets_used >= 8 {
                    break;
                }
            }

            if self.loop_video {
                self.decoder.flush();
                self.input_context.seek(0, ..i64::MAX).ok();
            } else {
                self.playing = false;
            }

            false
        }

        pub fn seek(&mut self, time_secs: f64) {
            let target = (time_secs / self.time_base) as i64;
            self.input_context.seek(target, ..i64::MAX).ok();
            self.decoder.flush();
        }

        pub fn restart(&mut self) {
            self.playing = true;
            self.decoder.flush();
            self.input_context.seek(0, ..i64::MAX).ok();
        }

        pub fn pixels(&self) -> &[u8] {
            &self.pixels
        }

        pub fn dimensions(&self) -> (u32, u32) {
            (self.width, self.height)
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    pub struct VideoPlayer {
        pub playing: bool,
        pub loop_video: bool,
        pub alpha: f32,
        width: u32,
        height: u32,
    }

    impl VideoPlayer {
        pub fn open(_path: &str) -> Result<Self, String> {
            log::warn!("VideoPlayer::open is not supported on web (stub)");
            Ok(Self {
                playing: false,
                loop_video: true,
                alpha: 1.0,
                width: 1,
                height: 1,
            })
        }

        pub fn update(&mut self, _dt: f32) -> bool {
            false
        }

        pub fn seek(&mut self, _time_secs: f64) {}

        pub fn restart(&mut self) {}

        pub fn pixels(&self) -> &[u8] {
            &[]
        }

        pub fn dimensions(&self) -> (u32, u32) {
            (self.width, self.height)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::VideoPlayer;

#[cfg(target_arch = "wasm32")]
pub use web::VideoPlayer;
