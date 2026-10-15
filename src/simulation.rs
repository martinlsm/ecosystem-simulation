mod berry;
mod constants;
mod hunger;
mod motion;
mod hare;
mod pathfinding;
mod tileset;
mod unit;

use std::f32::consts::PI;

use crate::{
    simulation::{constants::*, pathfinding::NavigationGrid, tileset::*, unit::fernworm, unit::zyrthid},
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
                motion::apply_rotation.run_if(in_state(AppState::Simulation)),
                hunger::hunger_drain.run_if(in_state(AppState::Simulation)),
                berry::spawn_berries.run_if(in_state(AppState::Simulation)),
                motion::update_velocity.run_if(in_state(AppState::Simulation)),
                motion::repel_bodies.run_if(in_state(AppState::Simulation)),
                hare::animate.run_if(in_state(AppState::Simulation)),
                hare::use_brain.run_if(in_state(AppState::Simulation)),
                unit::fernworm::use_brain.run_if(in_state(AppState::Simulation)),
                zyrthid::use_brain.run_if(in_state(AppState::Simulation)),
                (
                    unit::fernworm::eat_berries.run_if(in_state(AppState::Simulation)),
                    hare::eat_berries.run_if(in_state(AppState::Simulation)),
                    zyrthid::eat_fernworms.run_if(in_state(AppState::Simulation)),
                    hunger::kill_starved_units.run_if(in_state(AppState::Simulation)),
                    unit::kill_units.run_if(in_state(AppState::Simulation)),
                )
                    .chain(),
            ),
        )
        .add_observer(on_drag_move_screen)
        .insert_resource(SimData {
            num_berries: 0,
            max_berries: MAX_BERRIES,
        })
        .add_event::<unit::DeathEvent>();
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
                navigation_grid.set_blocked(
                    pathfinding::GridPosition { column, row },
                    true,
                );
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

    for _ in 0..NUM_FERNWORMS {
        let init_pos = Vec3::new(
            rng.gen_range(PLAYABLE_AREA_X0..PLAYABLE_AREA_X1) as f32,
            rng.gen_range(PLAYABLE_AREA_Y0..PLAYABLE_AREA_Y1) as f32,
            2.0,
        );

        commands.spawn((
            SimulationComponent,
            fernworm::Fernworm,
            unit::UnitType::Fernworm,
            motion::Rotation(0.0),
            Sprite {
                image: asset_server.load("sprites/fernworm.png"),
                ..default()
            },
            Transform {
                translation: init_pos,
                ..default()
            },
            motion::MovingBody {
                curr_velocity: Vec3::ZERO,
                max_speed: FERNWORM_MAX_SPEED,
                curr_acceleration: Vec3::ZERO,
                max_acceleration: 1000.0,
            },
            hunger::Hunger {
                curr_fullness: 100.0,
                max_fullness: 100.0,
                drain_per_unit_traveled: 25.0 / FERNWORM_MAX_SPEED,
                last_sampled_pos: init_pos,
            },
            motion::TargetPoint(None),
        ));
    }

    for _ in 0..NUM_ZYRTHIDS {
        let init_pos = Vec3::new(
            rng.gen_range(PLAYABLE_AREA_X0..PLAYABLE_AREA_X1) as f32,
            rng.gen_range(PLAYABLE_AREA_Y0..PLAYABLE_AREA_Y1) as f32,
            1.5,
        );

        commands.spawn((
            SimulationComponent,
            zyrthid::Zyrthid,
            unit::UnitType::Zyrthid,
            motion::Rotation(0.0),
            Sprite {
                image: asset_server.load("sprites/zyrthid.png"),
                ..default()
            },
            Transform {
                translation: init_pos,
                ..default()
            },
            motion::MovingBody {
                curr_velocity: Vec3::ZERO,
                max_speed: ZYRTHID_MAX_SPEED,
                curr_acceleration: Vec3::ZERO,
                max_acceleration: 1000.0,
            },
            hunger::Hunger {
                curr_fullness: 200.0,
                max_fullness: 200.0,
                drain_per_unit_traveled: 15.0 / ZYRTHID_MAX_SPEED,
                last_sampled_pos: init_pos,
            },
            motion::TargetPoint(None),
        ));
    }

    let hare_atlas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(HARE_SPRITE_WIDTH, HARE_SPRITE_HEIGHT),
        5,
        4,
        None,
        None,
    ));
    let hare_idle_atlas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(HARE_SPRITE_WIDTH, HARE_SPRITE_HEIGHT),
        4,
        4,
        None,
        None,
    ));
    let hare_idle_image = asset_server.load("sprites/animals/PNG/Without_shadow/Hare/Hare_Idle.png");
    let hare_walk_image = asset_server.load("sprites/animals/PNG/Without_shadow/Hare/Hare_Walk.png");

    for _ in 0..NUM_HARES {
        let init_pos = loop {
            let position = Vec3::new(
                rng.gen_range(PLAYABLE_AREA_X0..PLAYABLE_AREA_X1),
                rng.gen_range(PLAYABLE_AREA_Y0..PLAYABLE_AREA_Y1),
                2.0,
            );
            if !navigation_grid.is_blocked(position) {
                break position;
            }
        };

        commands.spawn((
            SimulationComponent,
            hare::Hare,
            hare::WalkAnimation {
                timer: Timer::from_seconds(0.12, TimerMode::Repeating),
                direction_row: 0,
                frame: 0,
                idle_image: hare_idle_image.clone(),
                idle_layout: hare_idle_atlas_layout.clone(),
                walk_image: hare_walk_image.clone(),
                walk_layout: hare_atlas_layout.clone(),
            },
            Sprite {
                image: hare_idle_image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: hare_idle_atlas_layout.clone(),
                    index: 0,
                }),
                ..default()
            },
            Transform::from_translation(init_pos),
            motion::MovingBody {
                curr_velocity: Vec3::ZERO,
                max_speed: HARE_MAX_SPEED,
                curr_acceleration: Vec3::ZERO,
                max_acceleration: 1000.0,
            },
            motion::TargetPoint(None),
        ));
    }

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
