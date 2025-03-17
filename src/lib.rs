use std::process::Termination;

use bevy::prelude::*;

#[unsafe(no_mangle)] // Safety: Only one `android_main` is defined.
#[cfg(target_os = "android")]
fn android_main(android_app: bevy::window::android_activity::AndroidApp) {
    _ = bevy::window::ANDROID_APP.set(android_app);
    _ = run();
}

#[unsafe(no_mangle)] // Safety: Only one `main_rs()` is defined.
#[cfg(target_os = "ios")]
extern "C" fn main_rs() {
    _ = run();
}

pub fn run() -> impl Termination {
    App::new()
        .add_plugins((
            DefaultPlugins,
            hephae::render::<(), ()>(),
            hephae::locales::<(), ()>(),
            hephae::text(),
            hephae::ui::<(), ()>(),
        ))
        .run()
}
