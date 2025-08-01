use bevy::prelude::States;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    Simulation,
}
