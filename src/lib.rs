#![feature(
    if_let_guard,
    impl_trait_in_assoc_type,
    let_chains,
    maybe_uninit_array_assume_init,
    maybe_uninit_slice
)]

use avian2d::prelude::*;
use bevy::prelude::*;
use hephae::prelude::*;
use mimalloc_redirect::MiMalloc;

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

pub fn run() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            hephae! { .. },
            ControlPlugins,
            StoragePlugin,
        ))
        .add_systems(Startup, on_startup)
        .run();
}

fn on_startup(mut commands: Commands) {
    commands.spawn(PrimaryCamera);
}
