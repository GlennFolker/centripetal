use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{input::MovementAction, world::collider::LevelCollider, MainCamera};

#[derive(Copy, Clone, Component)]
pub struct EditorCameraControl;

pub fn enter_editor(mut commands: Commands, camera: Query<Entity, With<MainCamera>>) {
    commands.spawn((TransformBundle::default(), LevelCollider::default()));

    let Ok(camera) = camera.get_single() else { return };
    let input = commands
        .spawn((
            EditorCameraControl,
            InputManagerBundle::with_map(
                InputMap::default().with_dual_axis(MovementAction::Move, KeyboardVirtualDPad::WASD),
            ),
        ))
        .id();

    commands.entity(camera).add_child(input);
}

pub fn exit_editor(mut commands: Commands, control: Query<Entity, With<EditorCameraControl>>) {
    let Ok(control) = control.get_single() else { return };
    commands.entity(control).despawn();
}
