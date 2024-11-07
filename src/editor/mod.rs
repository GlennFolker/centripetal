pub mod init;
pub mod ui;
pub mod widgets;

use bevy::prelude::*;

use crate::editor::{
    init::{enter_editor, exit_editor},
    widgets::EditorWidgetsPlugin,
};

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
            .add_systems(OnEnter(EditorState::On), enter_editor)
            .add_systems(OnExit(EditorState::On), exit_editor);
    }
}
