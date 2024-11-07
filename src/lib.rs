#[cfg(feature = "dev")]
pub mod editor;
pub mod input;
pub mod picking;
pub mod world;

use avian2d::prelude::*;
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_mod_picking::prelude::*;

#[cfg(feature = "dev")]
use crate::editor::EditorPlugin;
use crate::{picking::Avian2dBackend, world::WorldPlugin};

#[derive(Copy, Clone, Component)]
pub struct MainCamera;

#[derive(Copy, Clone, Component)]
pub struct RootUi;

#[inline(always)]
pub fn run() {
    App::new()
        .add_plugins(|app: &mut App| {
            app.add_plugins((
                DefaultPlugins,
                PhysicsPlugins::default(),
                DefaultPickingPlugins.build().add(Avian2dBackend),
                WorldPlugin,
            ))
            .add_systems(Startup, startup);

            #[cfg(feature = "dev")]
            app.add_plugins((PhysicsDebugPlugin::default(), EditorPlugin));
        })
        .run();
}

pub fn startup(mut commands: Commands) {
    commands.spawn((MainCamera, Camera2dBundle {
        camera: Camera { hdr: true, ..default() },
        projection: OrthographicProjection { scaling_mode: ScalingMode::FixedHorizontal(80.0), ..default() },
        ..default()
    }));

    commands.spawn((RootUi, NodeBundle {
        style: Style { margin: UiRect::all(Val::Percent(100.0)), padding: UiRect::all(Val::Px(0.0)), ..default() },
        ..default()
    }));
}
