use photon_engine::app::App;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    #[cfg(not(target_arch = "wasm32"))]
    {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "info");
        }
        env_logger::init();
        log::info!("PhotonEngine starting (native)...");
    }

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(async {
            let event_loop = winit::event_loop::EventLoop::new().unwrap();
            let mut app = App::new();
            event_loop.run_app(&mut app).expect("Event loop error");
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        let mut app = App::new();
        event_loop.run_app(&mut app).expect("Event loop error");
    }
}
