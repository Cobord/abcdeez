#[no_mangle]
#[cfg(target_os = "ios")]
// Called from ios-src/main.m, defined in ios-src/bindings.h
extern "C" fn main_rs() {
    use abcdeez_app::run;
    use winit::event_loop::EventLoop;
    let _ = run(EventLoop::with_user_event()).expect("App exited with error");
}

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    use abcdeez_app::run;
    use winit::platform::android::EventLoopBuilderExtAndroid;
    use winit::event_loop::EventLoop;

    let mut event_loop = EventLoop::with_user_event();
    event_loop.with_android_app(app);

    run(event_loop).expect("App exited with error");
}
