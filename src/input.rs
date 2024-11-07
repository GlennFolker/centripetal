use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
pub enum MovementAction {
    Move,
}

impl Actionlike for MovementAction {
    #[inline]
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            Self::Move => InputControlKind::DualAxis,
        }
    }
}
