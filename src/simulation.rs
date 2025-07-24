mod berry;
mod constants;
mod core;
mod fernworm;
mod hunger;
mod zyrthid;

use std::f32::consts::PI;

use crate::simulation::constants::*;
use crate::state::AppState;

use bevy::{
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
};
use rand::Rng;

#[derive(Resource)]
struct SimData {
    num_berries: u64,
    max_berries: u64,
}

#[derive(Component)]
struct SimulationComponent;

pub fn simulation_plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Simulation), setup)
        .add_systems(OnExit(AppState::Simulation), exit)
        .add_systems(Update, handle_input.run_if(in_state(AppState::Simulation)))
        .add_systems(
            Update,
            core::apply_velocity.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(
            Update,
            core::apply_rotation.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(
            Update,
            hunger::hunger_drain.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(
            Update,
            hunger::kill_starved_units.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(
            Update,
            berry::spawn_berries.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(
            Update,
            fernworm::eat_berries.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(
            Update,
            zyrthid::eat_fernworms.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(
            Update,
            core::update_velocity.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(
            Update,
            core::repel_bodies.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(
            Update,
            fernworm::use_brain.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(
            Update,
            zyrthid::use_brain.run_if(in_state(AppState::Simulation)),
        )
        .insert_resource(SimData {
            num_berries: 0,
            max_berries: MAX_BERRIES,
        });
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut game_data: ResMut<SimData>) {
    let mut rng = rand::thread_rng();

    commands.spawn((
        SimulationComponent,
        Sprite {
            image: asset_server.load("sprites/forest.png"),
            custom_size: Some(Vec2::new(SCREEN_WIDTH, SCREEN_HEIGHT)),
            ..default()
        },
    ));

    for _ in 0..NUM_FERNWORMS {
        let init_pos = Vec3::new(
            rng.gen_range(PLAYABLE_AREA_X0..PLAYABLE_AREA_X1) as f32,
            rng.gen_range(PLAYABLE_AREA_Y0..PLAYABLE_AREA_Y1) as f32,
            2.0,
        );

        commands.spawn((
            SimulationComponent,
            fernworm::Fernworm,
            core::Rotation(0.0),
            Sprite {
                image: asset_server.load("sprites/fernworm.png"),
                custom_size: Some(Vec2::new(FERNWORM_RENDER_WIDTH, FERNWORM_RENDER_HEIGHT)),
                ..default()
            },
            Transform {
                translation: init_pos,
                ..default()
            },
            core::MovingBody {
                curr_velocity: Vec3::ZERO,
                max_speed: 200.0,
                curr_acceleration: Vec3::ZERO,
                max_acceleration: 1000.0,
            },
            hunger::Hunger {
                curr_fullness: 100.0,
                max_fullness: 100.0,
                drain_per_unit_traveled: 5.0 / 200.0,
                last_sampled_pos: init_pos,
            },
            core::TargetPoint(None),
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
            core::Rotation(0.0),
            Sprite {
                image: asset_server.load("sprites/zyrthid.png"),
                custom_size: Some(Vec2::new(ZYRTHID_RENDER_WIDTH, ZYRTHID_RENDER_HEIGHT)),
                ..default()
            },
            Transform {
                translation: init_pos,
                ..default()
            },
            core::MovingBody {
                curr_velocity: Vec3::ZERO,
                max_speed: 150.0,
                curr_acceleration: Vec3::ZERO,
                max_acceleration: 1000.0,
            },
            hunger::Hunger {
                curr_fullness: 200.0,
                max_fullness: 200.0,
                drain_per_unit_traveled: 5.0 / 250.0,
                last_sampled_pos: init_pos,
            },
            core::TargetPoint(None),
        ));
    }

    game_data.num_berries = 0;
}

fn handle_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut state: ResMut<NextState<AppState>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        state.set(AppState::Menu);
    }
}

fn exit(query: Query<Entity, With<SimulationComponent>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
