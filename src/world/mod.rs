pub mod collider;

use bevy::prelude::*;

use crate::world::collider::update_level_collider;

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_level_collider);
    }
}
