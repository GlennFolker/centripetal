use bevy::{app::PluginGroupBuilder, prelude::*};
use leafwing_input_manager::{plugin::InputManagerSystem, prelude::*};

#[derive(Actionlike, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Reflect)]
#[reflect(Debug)]
pub enum Attack {
    Primary,
    Secondary,
}

#[derive(Actionlike, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Reflect)]
#[actionlike(DualAxis)]
#[reflect(Debug)]
pub struct Jump;

#[derive(Actionlike, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Reflect)]
#[actionlike(DualAxis)]
#[reflect(Debug)]
pub struct Dash;

#[derive(Actionlike, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Reflect)]
#[actionlike(DualAxis)]
#[reflect(Debug)]
pub struct Move;

#[derive(Actionlike, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Reflect)]
#[reflect(Debug)]
pub enum Controller {
    Primary,
    Secondary,
    Jump,
    Dash,
    #[actionlike(DualAxis)]
    Move,
}

#[derive(Component, Copy, Clone, Default)]
#[require(ActionState<Controller>, InputMap<Controller>(Self::default_map))]
pub struct Player;
impl Player {
    fn default_map() -> InputMap<Controller> {
        InputMap::new([
            (Controller::Primary, KeyCode::KeyH),
            (Controller::Secondary, KeyCode::KeyJ),
            (Controller::Jump, KeyCode::Space),
            (Controller::Dash, KeyCode::ShiftLeft),
        ])
        .with_dual_axis(Controller::Move, VirtualDPad::wasd())
    }
}

pub struct ControlPlugins;
impl PluginGroup for ControlPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(InputManagerPlugin::<Attack>::default())
            .add(InputManagerPlugin::<Jump>::default())
            .add(InputManagerPlugin::<Dash>::default())
            .add(InputManagerPlugin::<Move>::default())
            .add(InputManagerPlugin::<Controller>::default())
            .add(|app: &mut App| {
                app.add_systems(
                    PreUpdate,
                    (copy_attack_state, copy_jump_state, copy_dash_state, copy_move_state)
                        .in_set(InputManagerSystem::ManualControl),
                );
            })
    }
}

fn copy_attack_state(mut query: Query<(&ActionState<Controller>, &mut ActionState<Attack>)>) {
    for (control, mut attack) in &mut query {
        if let Some(primary) = control.button_data(&Controller::Primary) {
            attack.set_button_data(Attack::Primary, primary.clone())
        }

        if let Some(secondary) = control.button_data(&Controller::Secondary) {
            attack.set_button_data(Attack::Secondary, secondary.clone())
        }
    }
}

fn copy_jump_state(mut query: Query<(&ActionState<Controller>, &mut ActionState<Jump>)>) {
    for (control, mut jump) in &mut query {
        let Some(state) = control.button_data(&Controller::Jump) else { continue };
        jump.set_button_data(Jump, state.clone())
    }
}

fn copy_dash_state(mut query: Query<(&ActionState<Controller>, &mut ActionState<Dash>)>) {
    for (control, mut dash) in &mut query {
        let Some(state) = control.button_data(&Controller::Dash) else { continue };
        dash.set_button_data(Dash, state.clone())
    }
}

fn copy_move_state(mut query: Query<(&ActionState<Controller>, &mut ActionState<Move>)>) {
    for (control, mut mover) in &mut query {
        if let Some(dir) = control.dual_axis_data(&Controller::Move) {
            mover.set_axis_pair(&Move, dir.pair)
        }
    }
}
