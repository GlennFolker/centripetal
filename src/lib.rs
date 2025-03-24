use avian2d::prelude::*;
use bevy::prelude::*;
use hephae::prelude::*;
#[cfg(not(target_family = "wasm"))]
use mimalloc_redirect::MiMalloc;

#[cfg(not(target_family = "wasm"))]
#[global_allocator]
static ALLOC: MiMalloc = MiMalloc;

pub mod persist;
pub mod player;

#[derive(Component, Copy, Clone, Default)]
#[require(Camera2d)]
pub struct PrimaryCamera;

#[cfg_attr(target_family = "wasm", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn run() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default(), hephae! {
            ..
        }))
        .add_systems(Startup, on_startup)
        .run();
}

fn on_startup(mut commands: Commands) {
    commands.spawn(PrimaryCamera);
}
