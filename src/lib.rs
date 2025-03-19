use std::process::Termination;

use avian2d::PhysicsPlugins;
use bevy::prelude::*;
use mimalloc_redirect::MiMalloc;

#[global_allocator]
static ALLOC: MiMalloc = MiMalloc;

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

#[derive(Component, Copy, Clone, Default)]
#[require(Camera2d)]
pub struct PrimaryCamera;

#[inline]
pub fn run() -> impl Termination {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Centripetal".into(),
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
            hephae::render::<(), ()>(),
            hephae::locales::<(), ()>(),
            hephae::text(),
            hephae::ui::<(), ()>(),
        ))
        .add_systems(Startup, on_startup)
        .run()
}

fn on_startup(mut commands: Commands) {
    commands.spawn(PrimaryCamera);
}
