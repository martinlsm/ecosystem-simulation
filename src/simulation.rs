use std::f32::consts::PI;

use crate::state::AppState;

use bevy::{
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
};
use rand::Rng;

const NUM_HERBIVORES: usize = 3;
const MAX_BERRIES: u64 = 10;

const HERBIVORE_SPRITE_HEIGHT: f32 = 16.0;
const HERBIVORE_SPRITE_WIDTH: f32 = 7.0;
const HERBIVORE_SCALE_FACTOR: f32 = 4.0;
const HERBIVORE_RENDER_HEIGHT: f32 = HERBIVORE_SPRITE_HEIGHT * HERBIVORE_SCALE_FACTOR;
const HERBIVORE_RENDER_WIDTH: f32 = HERBIVORE_SPRITE_WIDTH * HERBIVORE_SCALE_FACTOR;

#[derive(Resource)]
struct SimData {
    num_berries: u64,
    max_berries: u64,
}

#[derive(Component)]
struct MovingBody {
    curr_velocity: Vec3,
    max_speed: f32,
    curr_acceleration: Vec3,
    max_acceleration: f32,
}

#[derive(Component)]
struct SimulationComponent;

#[derive(Component)]
struct Rotation(f32);

#[derive(Component)]
struct Herbivore;

#[derive(Component)]
struct Berry;

#[derive(Component)]
struct TargetPoint(Vec3);

pub fn simulation_plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Simulation), setup)
        .add_systems(OnExit(AppState::Simulation), exit)
        .add_systems(Update, handle_input.run_if(in_state(AppState::Simulation)))
        .add_systems(
            Update,
            apply_velocity.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(Update, spawn_berries.run_if(in_state(AppState::Simulation)))
        .add_systems(Update, eat_berries.run_if(in_state(AppState::Simulation)))
        .add_systems(
            Update,
            update_velocity.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(Update, repel_bodies.run_if(in_state(AppState::Simulation)))
        .add_systems(Update, use_brain.run_if(in_state(AppState::Simulation)))
        .insert_resource(SimData {
            num_berries: 0,
            max_berries: MAX_BERRIES,
        });
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut rng = rand::thread_rng();

    for _ in 0..NUM_HERBIVORES {
        let init_pos = Vec3::new(
            400.0 * rng.gen::<f32>() - 200.0,
            400.0 * rng.gen::<f32>() - 200.0,
            2.0,
        );

        commands.spawn((
            SimulationComponent,
            Herbivore,
            Rotation(0.0),
            Sprite {
                image: asset_server.load("sprites/herbivore.png"),
                custom_size: Some(Vec2::new(HERBIVORE_RENDER_WIDTH, HERBIVORE_RENDER_HEIGHT)),
                ..default()
            },
            Transform {
                translation: init_pos,
                ..default()
            },
            MovingBody {
                curr_velocity: Vec3::ZERO,
                max_speed: 200.0,
                curr_acceleration: Vec3::ZERO,
                max_acceleration: 1000.0,
            },
            TargetPoint(bevy::math::vec3(0.0, 0.0, 0.0)),
        ));
    }
}

fn handle_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut state: ResMut<NextState<AppState>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        state.set(AppState::Menu);
    }
}

fn apply_velocity(mut query: Query<(&mut Rotation, &mut Transform, &MovingBody)>, time: Res<Time>) {
    for (mut rotation, mut transform, moving_body) in query.iter_mut() {
        // Update position.
        transform.translation =
            transform.translation + moving_body.curr_velocity * time.delta_secs();

        // Update sprite rotation.
        let velocity = &moving_body.curr_velocity;
        if velocity.length() > 0.001 {
            let mut angle = (velocity.y / velocity.x).atan() - PI / 2.0;
            if velocity.x < 0.0 {
                angle += PI;
            }

            rotation.0 = angle;
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

fn update_velocity(mut query: Query<(&mut MovingBody, &TargetPoint)>, time: Res<Time>) {
    for (mut moving_body, target_point) in query.iter_mut() {
        moving_body.curr_acceleration = (target_point.0.normalize_or_zero()
            * moving_body.max_acceleration)
            .clamp_length_max(time.delta_secs() * moving_body.max_acceleration);
        moving_body.curr_velocity = (moving_body.curr_velocity + moving_body.curr_acceleration)
            .clamp_length_max(moving_body.max_speed);
    }
}

fn spawn_berries(
    mut game_data: ResMut<SimData>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::thread_rng();

    for _ in game_data.num_berries..game_data.max_berries {
        let init_pos_berry = Vec3::new(
            400.0 * rng.gen::<f32>() - 200.0,
            400.0 * rng.gen::<f32>() - 200.0,
            1.0,
        );

        commands.spawn((
            SimulationComponent,
            Berry,
            Sprite {
                image: asset_server.load("sprites/berry.png"),
                custom_size: Some(Vec2::new(16.0, 16.0)),
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

fn use_brain(
    mut herbivore_query: Query<
        (&Transform, &MovingBody, &mut TargetPoint),
        (With<Herbivore>, Without<Berry>),
    >,
    berry_query: Query<&Transform, (With<Berry>, Without<Herbivore>)>,
) {
    for (herbivore_transform, velocity, mut herbivore_target_point) in herbivore_query.iter_mut() {
        let mut min_dist = f32::MAX;
        let mut target_berry_pos: Option<Vec3> = None;

        for berry_transform in berry_query.iter() {
            let dist =
                (berry_transform.translation - herbivore_transform.translation).length_squared();
            if dist < min_dist {
                min_dist = dist;
                target_berry_pos = Some(berry_transform.translation);
            }
        }

        match target_berry_pos {
            Some(target_pos) => {
                // Algorithm taken from: https://gamedev.stackexchange.com/questions/17313/how-does-one-prevent-homing-missiles-from-orbiting-their-targets
                let v_targ = -velocity.curr_velocity;
                let s = herbivore_transform.translation - target_pos;
                let t_estimate = s.length() / (v_targ.length() + f32::EPSILON);

                // Unclear why this constant makes things better, but it prevents herbivores from
                // oscillating to the left and right when chasing after berries.
                let stability_constant = 0.8;
                let target = target_pos + stability_constant * v_targ * t_estimate
                    - herbivore_transform.translation;
                herbivore_target_point.0 = target;
            }
            None => herbivore_target_point.0 = Vec3::ZERO,
        }
    }
}

fn eat_berries(
    mut commands: Commands,
    mut game_data: ResMut<SimData>,
    berry_query: Query<(Entity, &mut Transform), (With<Berry>, Without<Herbivore>)>,
    herbivore_query: Query<(&mut Transform, &Rotation), (With<Herbivore>, Without<Berry>)>,
) {
    for (berry_entity, berry_transform) in berry_query.iter() {
        let berry_size = berry_transform.scale.truncate() * Vec2::new(2.0, 2.0);

        for (herbivore_transform, rotation) in herbivore_query.iter() {
            // TODO: Extract these parameters to some form of config file/code.
            let mouth_offset = herbivore_transform.scale.truncate() * 5.0;
            let mouth_translation = herbivore_transform.translation
                + Vec3::new(
                    -rotation.0.sin() * mouth_offset.x,
                    rotation.0.cos() * mouth_offset.y,
                    0.0,
                );
            let mouth_size = herbivore_transform.scale.truncate() * Vec2::new(4.0, 2.0);

            let berry = Aabb2d::new(berry_transform.translation.truncate(), berry_size);
            let mouth = Aabb2d::new(mouth_translation.truncate(), mouth_size);

            if berry.intersects(&mouth) {
                commands.entity(berry_entity).despawn();
                game_data.num_berries -= 1;

                // Break here so that no other herbivore can eat the same berry during the same frame.
                break;
            }
        }
    }
}

fn repel_bodies(mut body_query: Query<(&mut Transform, &mut MovingBody)>, time: Res<Time>) {
    let mut combinations = body_query.iter_combinations_mut();
    while let Some([mut t1, mut t2]) = combinations.fetch_next() {
        // Bounds before collision force is applied.
        const COLLISION_RADIUS_SQUARED: f32 = (HERBIVORE_RENDER_HEIGHT
            + HERBIVORE_RENDER_WIDTH / 2.0)
            * (HERBIVORE_RENDER_HEIGHT + HERBIVORE_RENDER_WIDTH / 2.0);

        // Strength of the collision force.
        const FORCE_CONSTANT: f32 = 500000.0;

        let p1: Vec3 = t1.0.as_mut().translation;
        let p2: Vec3 = t2.0.as_mut().translation;
        let squared_dist = p1.distance_squared(p2);

        let force = if squared_dist < COLLISION_RADIUS_SQUARED {
            FORCE_CONSTANT / (squared_dist + f32::EPSILON)
        } else {
            0.0
        };

        let body1_push_dir: Vec3 = (p1 - p2).normalize_or_zero() * force * time.delta_secs();
        let body2_push_dir: Vec3 = (p1 - p2).normalize_or_zero() * force * time.delta_secs();

        let moving_body1 = t1.1.as_mut();
        moving_body1.curr_velocity += body1_push_dir;

        let body2 = t1.1.as_mut();
        body2.curr_velocity += body2_push_dir;
    }
}

fn exit(query: Query<Entity, With<SimulationComponent>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
