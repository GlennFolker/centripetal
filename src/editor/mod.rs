pub mod init;
pub mod widgets;

use bevy::prelude::*;

use crate::editor::{init::initial_scene, widgets::EditorWidgetsPlugin};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, States)]
pub enum EditorState {
    Off,
    #[default]
    On,
}

pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EditorWidgetsPlugin)
            .init_state::<EditorState>()
            .add_systems(OnEnter(EditorState::On), initial_scene);
    }
}
