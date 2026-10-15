use bevy::prelude::*;

use crate::simulation::*;

#[derive(Component)]
pub struct Berry;

pub fn spawn_berries(
    mut game_data: ResMut<SimData>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    navigation_grid: Res<pathfinding::NavigationGrid>,
) {
    let mut rng = rand::thread_rng();

    for _ in game_data.num_berries..game_data.max_berries {
        let init_pos_berry = loop {
            let position = Vec3::new(
                rng.gen_range(PLAYABLE_AREA_X0..PLAYABLE_AREA_X1),
                rng.gen_range(PLAYABLE_AREA_Y0..PLAYABLE_AREA_Y1),
                1.0,
            );
            if !navigation_grid.is_blocked(position) {
                break position;
            }
        };

        commands.spawn((
            SimulationComponent,
            Berry,
            Sprite {
                image: asset_server.load("sprites/berry.png"),
                ..default()
            },
            Transform {
                translation: init_pos_berry,
                ..default()
            },
        ));
    }

    game_data.num_berries = game_data.max_berries;
}
