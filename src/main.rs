mod menu;
mod simulation;
mod state;

use state::AppState;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_state::<AppState>()
        .add_systems(Startup, setup)
        .add_plugins((menu::menu_plugin, simulation::simulation_plugin))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}