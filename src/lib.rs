#![feature(if_let_guard, impl_trait_in_assoc_type)]

use avian2d::prelude::*;
use bevy::prelude::*;
use hephae::prelude::*;
#[cfg(not(target_family = "wasm"))]
use mimalloc_redirect::MiMalloc;

#[cfg(not(target_family = "wasm"))]
#[global_allocator]
static ALLOC: MiMalloc = MiMalloc;

mod control;
mod storage;
pub use control::*;
pub use storage::*;

pub mod persist;

#[derive(Component, Copy, Clone, Default, Debug)]
#[require(Camera2d)]
pub struct PrimaryCamera;

#[cfg_attr(target_family = "wasm", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn run() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default(), hephae! { .. }, ControlPlugins))
        .add_systems(Startup, on_startup)
        .run();
}

fn on_startup(mut commands: Commands) {
    commands.spawn(PrimaryCamera);
}
