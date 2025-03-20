use std::process::Termination;

use avian2d::PhysicsPlugins;
use bevy::prelude::*;
use mimalloc_redirect::MiMalloc;

pub mod persist;

#[global_allocator]
static ALLOC: MiMalloc = MiMalloc;

#[derive(Component, Copy, Clone, Default)]
#[require(Camera2d)]
pub struct PrimaryCamera;

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
