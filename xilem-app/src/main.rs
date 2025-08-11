// Copyright 2024 ABCDEEZ
// Main entry point for the ABCDEEZ Xilem learning application

fn main() {
    abcdeez_app::run(winit::event_loop::EventLoop::with_user_event()).expect("App exited with error");
}