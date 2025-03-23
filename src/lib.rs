use avian2d::prelude::*;
use bevy::{log::LogPlugin, prelude::*};
#[cfg(not(target_family = "wasm"))]
use mimalloc_redirect::MiMalloc;

#[cfg(not(target_family = "wasm"))]
#[global_allocator]
static ALLOC: MiMalloc = MiMalloc;

pub mod persist;

#[derive(Component, Copy, Clone, Default)]
#[require(Camera2d)]
pub struct PrimaryCamera;

#[cfg_attr(target_family = "wasm", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn run() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().add_after::<LogPlugin>(|_: &mut App| {
                #[cfg(not(target_family = "wasm"))]
                info!("Using MiMalloc {}", MiMalloc::get_version())
            }),
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
