pub mod collider;

use bevy::prelude::*;

use crate::editor::{
    widgets::collider::{draw_collider_widget, update_collider_widget},
    EditorState,
};

pub struct EditorWidgetsPlugin;
impl Plugin for EditorWidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_collider_widget.run_if(in_state(EditorState::On)))
            .add_systems(
                PostUpdate,
                draw_collider_widget
                    .run_if(in_state(EditorState::On))
                    .after_ignore_deferred(TransformSystem::TransformPropagate),
            );
    }
}
