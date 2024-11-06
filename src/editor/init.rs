use bevy::prelude::*;

use crate::world::collider::LevelCollider;

pub fn initial_scene(mut commands: Commands) {
    commands.spawn((TransformBundle::default(), LevelCollider::default()));
}
