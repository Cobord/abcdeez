use graph_learning_app::run;
use winit::event_loop::EventLoop;

pub fn main() {
    run(EventLoop::with_user_event());
}
