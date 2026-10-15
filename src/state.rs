use bevy::prelude::{Resource, States};

#[derive(Resource)]
pub struct SimulationSeed(pub u64);

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    Simulation,
}
