mod berry;
mod constants;
mod hare;
mod motion;
mod pathfinding;
mod tileset;

use crate::{
    simulation::{constants::*, pathfinding::NavigationGrid, tileset::*},
    state::{AppState, SimulationSeed},
};

use bevy::{
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
};
use rand::{rngs::StdRng, Rng, SeedableRng};

#[derive(Resource)]
struct SimData {
    num_berries: u64,
    max_berries: u64,
}

#[derive(Component)]
struct SimulationComponent;

#[derive(Component)]
pub(crate) struct MapRock;

pub fn simulation_plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Simulation), setup)
        .add_systems(OnExit(AppState::Simulation), exit)
        .add_systems(
            Update,
            (
                handle_input.run_if(in_state(AppState::Simulation)),
                motion::apply_velocity.run_if(in_state(AppState::Simulation)),
                berry::spawn_berries.run_if(in_state(AppState::Simulation)),
                hare::decision_system.run_if(in_state(AppState::Simulation)),
                motion::update_velocity.run_if(in_state(AppState::Simulation)),
                motion::repel_bodies.run_if(in_state(AppState::Simulation)),
                hare::animate.run_if(in_state(AppState::Simulation)),
                (hare::eat_berries.run_if(in_state(AppState::Simulation)),).chain(),
            )
            .chain(),
        )
        .add_observer(on_drag_move_screen)
        .insert_resource(SimData {
            num_berries: 0,
            max_berries: MAX_BERRIES,
        });
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut game_data: ResMut<SimData>,
    seed: Res<SimulationSeed>,
) {
    let mut rng = StdRng::seed_from_u64(seed.0);
    let tileset = asset_server.load("tileset/Tileset_32x32.png");
    let atlas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(32),
        16,
        16,
        None,
        None,
    ));
    let mut navigation_grid = NavigationGrid::new();

    for row in 0..MAP_ROWS {
        for column in 0..MAP_COLUMNS {
            let is_edge =
                row == 0 || row == MAP_ROWS - 1 || column == 0 || column == MAP_COLUMNS - 1;
            let is_rock = is_edge || rng.gen_bool(0.1);
            let tile_position = Vec3::new(
                -MAP_WIDTH / 2.0 + TILE_WIDTH * (column as f32 + 0.5),
                -MAP_HEIGHT / 2.0 + TILE_HEIGHT * (row as f32 + 0.5),
                0.0,
            );

            commands.spawn((
                SimulationComponent,
                Sprite {
                    image: tileset.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas_layout.clone(),
                        index: GRASS_TILE_INDEX,
                    }),
                    ..default()
                },
                Transform::from_translation(tile_position),
            ));

            if is_rock {
                navigation_grid.set_blocked(pathfinding::GridPosition { column, row }, true);
                commands.spawn((
                    SimulationComponent,
                    MapRock,
                    Sprite {
                        image: tileset.clone(),
                        texture_atlas: Some(TextureAtlas {
                            layout: atlas_layout.clone(),
                            index: ROCK_TILE_INDEX,
                        }),
                        ..default()
                    },
                    Transform::from_translation(tile_position + Vec3::Z * 0.1),
                ));
            }
        }
    }

    hare::spawn(
        &mut commands,
        &asset_server,
        &mut texture_atlas_layouts,
        &mut rng,
        &navigation_grid,
    );

    commands.insert_resource(navigation_grid);
    game_data.num_berries = 0;
}

fn exit(query: Query<Entity, With<SimulationComponent>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut state: ResMut<NextState<AppState>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        state.set(AppState::Menu);
    }
}

fn on_drag_move_screen(
    drag: Trigger<Pointer<Drag>>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
    state: Res<State<AppState>>,
) {
    // Currently no way to only trigger the observer in one state. We have to explicitly check the state here.
    if *state != AppState::Simulation {
        return;
    }

    camera.translation.x -= drag.delta.x;
    camera.translation.y += drag.delta.y;
}
