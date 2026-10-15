mod menu;
mod simulation;
mod state;

use state::{AppState, SimulationSeed};

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_state::<AppState>()
        .insert_resource(SimulationSeed(42))
        .add_systems(Startup, setup)
        .add_plugins((menu::menu_plugin, simulation::simulation_plugin))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scale: 0.3,
            ..OrthographicProjection::default_2d()
        }),
    ));
}
