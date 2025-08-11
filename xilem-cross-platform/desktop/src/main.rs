use abcdeez_app::run;

pub fn main() {
    run(winit::event_loop::EventLoop::with_user_event()).expect("App exited with error");
}
