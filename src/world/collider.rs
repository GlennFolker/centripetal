use avian2d::prelude::*;
use bevy::{math::vec2, prelude::*};

#[derive(Component, Clone, Reflect)]
#[reflect(Component)]
pub struct LevelCollider {
    pub vertices: Vec<Vec2>,
}

impl Default for LevelCollider {
    #[inline]
    fn default() -> Self {
        Self {
            vertices: vec![vec2(-5.0, -5.0), vec2(5.0, -5.0), vec2(5.0, 5.0), vec2(-5.0, 5.0)],
        }
    }
}

impl From<&LevelCollider> for Collider {
    fn from(value: &LevelCollider) -> Self {
        let vertices = value.vertices.clone();
        let indices = (0..vertices.len() as u32)
            .map(|i| [i, (i + 1) % vertices.len() as u32])
            .collect();

        Self::polyline(vertices, Some(indices))
    }
}

pub fn update_level_collider(mut commands: Commands, colliders: Query<(Entity, &LevelCollider), Changed<LevelCollider>>) {
    for (e, collider) in &colliders {
        commands.entity(e).insert((RigidBody::Static, Collider::from(collider)));
    }
}
