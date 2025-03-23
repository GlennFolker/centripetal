use avian2d::prelude::*;
use bevy::{log::LogPlugin, prelude::*};
use mimalloc_redirect::MiMalloc;

pub mod persist;

include!(concat!(env!("OUT_DIR"), "/asset_directory.rs"));

#[global_allocator]
static ALLOC: MiMalloc = MiMalloc;

#[derive(Component, Copy, Clone, Default)]
#[require(Camera2d)]
pub struct PrimaryCamera;

#[cfg_attr(target_family = "wasm", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn run() {
    #[cfg(target_family = "wasm")]
    console_log::init().expect("Couldn't initialize logger");

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: ASSET_DIRECTORY.into(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Centripetal".into(),
                        ..default()
                    }),
                    ..default()
                })
                .add_after::<LogPlugin>(|_: &mut App| info!("Using MiMalloc {}", MiMalloc::get_version())),
            PhysicsPlugins::default(),
            hephae::render::<(), ()>(),
            hephae::locales::<(), ()>(),
            hephae::text(),
            hephae::ui::<(), ()>(),
        ))
        .add_systems(Startup, on_startup)
        .run();
}

fn on_startup(mut commands: Commands) {
    commands.spawn(PrimaryCamera);
}
