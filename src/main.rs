use photon_engine::app::App;

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        web_sys::console::log_1(&"PhotonEngine: main() started".into());
    }

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
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        web_sys::console::log_1(&"PhotonEngine: EventLoop created, calling spawn_app".into());
        use winit::platform::web::EventLoopExtWebSys;
        let _ = event_loop.spawn_app(App::new());
        web_sys::console::log_1(&"PhotonEngine: spawn_app returned".into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        let mut app = App::new();
        event_loop.run_app(&mut app).expect("Event loop error");
    }
}
