use std::f32::consts::PI;

use crate::state::AppState;

use bevy::{
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
};
use rand::Rng;

const NUM_ZYRTHIDS: usize = 1;
const NUM_FERNWORMS: usize = 10;
const MAX_BERRIES: u64 = 10;

const BERRY_FULLNESS_GAIN: f32 = 40.0;
const FERNWORM_FULLNESS_GAIN: f32 = 80.0;

const BERRY_RENDER_HEIGHT: f32 = 16.0;
const BERRY_RENDER_WIDTH: f32 = 16.0;

const FERNWORM_SPRITE_HEIGHT: u32 = 16;
const FERNWORM_SPRITE_WIDTH: u32 = 7;
const FERNWORM_SCALE_FACTOR: f32 = 4.0;
const FERNWORM_RENDER_HEIGHT: f32 = (FERNWORM_SPRITE_HEIGHT as f32) * FERNWORM_SCALE_FACTOR;
const FERNWORM_RENDER_WIDTH: f32 = (FERNWORM_SPRITE_WIDTH as f32) * FERNWORM_SCALE_FACTOR;

const ZYRTHID_SPRITE_HEIGHT: u32 = 25;
const ZYRTHID_SPRITE_WIDTH: u32 = 11;
const ZYRTHID_SCALE_FACTOR: f32 = 4.0;
const ZYRTHID_RENDER_HEIGHT: f32 = (ZYRTHID_SPRITE_HEIGHT as f32) * ZYRTHID_SCALE_FACTOR;
const ZYRTHID_RENDER_WIDTH: f32 = (ZYRTHID_SPRITE_WIDTH as f32) * ZYRTHID_SCALE_FACTOR;

const SCREEN_WIDTH: f32 = 1920.0;
const SCREEN_HEIGHT: f32 = 1080.0;
const PLAYABLE_AREA_X0: f32 = -(SCREEN_WIDTH / 2.0 - 350.0);
const PLAYABLE_AREA_X1: f32 = SCREEN_WIDTH / 2.0 - 350.0;
const PLAYABLE_AREA_Y0: f32 = -(SCREEN_HEIGHT / 2.0 - 200.0);
const PLAYABLE_AREA_Y1: f32 = SCREEN_HEIGHT / 2.0 - 200.0;

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
struct Hunger {
    curr_fullness: f32,
    max_fullness: f32,
    drain_per_unit_traveled: f32,
    last_sampled_pos: Vec3,
}

#[derive(Component)]
struct SimulationComponent;

#[derive(Component)]
struct Rotation(f32);

#[derive(Component)]
struct Fernworm;

#[derive(Component)]
struct Zyrthid;

#[derive(Component)]
struct FernwormCorpse;

#[derive(Component)]
struct Berry;

#[derive(Component)]
struct TargetPoint(Option<Vec3>);

pub fn simulation_plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Simulation), setup)
        .add_systems(OnExit(AppState::Simulation), exit)
        .add_systems(Update, handle_input.run_if(in_state(AppState::Simulation)))
        .add_systems(
            Update,
            apply_velocity.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(
            Update,
            apply_rotation.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(Update, hunger_drain.run_if(in_state(AppState::Simulation)))
        .add_systems(
            Update,
            kill_starved_units.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(Update, spawn_berries.run_if(in_state(AppState::Simulation)))
        .add_systems(Update, eat_berries.run_if(in_state(AppState::Simulation)))
        .add_systems(Update, eat_fernworms.run_if(in_state(AppState::Simulation)))
        .add_systems(
            Update,
            update_velocity.run_if(in_state(AppState::Simulation)),
        )
        .add_systems(Update, repel_bodies.run_if(in_state(AppState::Simulation)))
        .add_systems(Update, use_brain.run_if(in_state(AppState::Simulation)))
        .add_systems(
            Update,
            use_zyrthid_brain.run_if(in_state(AppState::Simulation)),
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
            Fernworm,
            Rotation(0.0),
            Sprite {
                image: asset_server.load("sprites/fernworm.png"),
                custom_size: Some(Vec2::new(FERNWORM_RENDER_WIDTH, FERNWORM_RENDER_HEIGHT)),
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
            Hunger {
                curr_fullness: 100.0,
                max_fullness: 100.0,
                drain_per_unit_traveled: 5.0 / 200.0,
                last_sampled_pos: init_pos,
            },
            TargetPoint(None),
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
            Zyrthid,
            Rotation(0.0),
            Sprite {
                image: asset_server.load("sprites/zyrthid.png"),
                custom_size: Some(Vec2::new(ZYRTHID_RENDER_WIDTH, ZYRTHID_RENDER_HEIGHT)),
                ..default()
            },
            Transform {
                translation: init_pos,
                ..default()
            },
            MovingBody {
                curr_velocity: Vec3::ZERO,
                max_speed: 150.0,
                curr_acceleration: Vec3::ZERO,
                max_acceleration: 1000.0,
            },
            Hunger {
                curr_fullness: 200.0,
                max_fullness: 200.0,
                drain_per_unit_traveled: 5.0 / 250.0,
                last_sampled_pos: init_pos,
            },
            TargetPoint(None),
        ));
    }

    game_data.num_berries = 0;
}

fn handle_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut state: ResMut<NextState<AppState>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        state.set(AppState::Menu);
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &MovingBody)>, time: Res<Time>) {
    for (mut transform, moving_body) in query.iter_mut() {
        // Update position.
        transform.translation =
            transform.translation + moving_body.curr_velocity * time.delta_secs();
    }
}

fn apply_rotation(mut query: Query<(&mut Rotation, &mut Transform, &MovingBody)>) {
    for (mut rotation, mut transform, moving_body) in query.iter_mut() {
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
        if let Some(p) = target_point.0 {
            moving_body.curr_acceleration = (p.normalize_or_zero() * moving_body.max_acceleration)
                .clamp_length_max(time.delta_secs() * moving_body.max_acceleration);
            moving_body.curr_velocity = (moving_body.curr_velocity + moving_body.curr_acceleration)
                .clamp_length_max(moving_body.max_speed);
        } else {
            moving_body.curr_acceleration = -moving_body.curr_velocity
        }
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
            rng.gen_range(PLAYABLE_AREA_X0..PLAYABLE_AREA_X1) as f32,
            rng.gen_range(PLAYABLE_AREA_Y0..PLAYABLE_AREA_Y1) as f32,
            1.0,
        );

        commands.spawn((
            SimulationComponent,
            Berry,
            Sprite {
                image: asset_server.load("sprites/berry.png"),
                custom_size: Some(Vec2::new(BERRY_RENDER_WIDTH, BERRY_RENDER_HEIGHT)),
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
    mut fernworm_query: Query<(&Transform, &MovingBody, &mut TargetPoint), With<Fernworm>>,
    berry_query: Query<&Transform, With<Berry>>,
) {
    for (fernworm_transform, moving_body, mut fernworm_target_point) in fernworm_query.iter_mut() {
        let mut min_dist = f32::MAX;
        let mut target_berry_pos: Option<Vec3> = None;

        for berry_transform in berry_query.iter() {
            let dist =
                (berry_transform.translation - fernworm_transform.translation).length_squared();
            if dist < min_dist {
                min_dist = dist;
                target_berry_pos = Some(berry_transform.translation);
            }
        }

        match target_berry_pos {
            Some(target_pos) => {
                // Algorithm taken from: https://gamedev.stackexchange.com/questions/17313/how-does-one-prevent-homing-missiles-from-orbiting-their-targets
                let v_targ = -moving_body.curr_velocity;
                let s = fernworm_transform.translation - target_pos;
                let t_estimate = s.length() / (v_targ.length() + f32::EPSILON);

                // Unclear why this constant makes things better, but it
                // prevents oscillating to the left and right when chasing after
                // targets.
                let stability_constant = 0.8;
                let target = target_pos + stability_constant * v_targ * t_estimate
                    - fernworm_transform.translation;
                fernworm_target_point.0 = Some(target);
            }
            None => fernworm_target_point.0 = None,
        }
    }
}

fn use_zyrthid_brain(
    mut zyrthid_query: Query<(&Transform, &MovingBody, &mut TargetPoint), (With<Zyrthid>,)>,
    fernworm_query: Query<&Transform, With<Fernworm>>,
) {
    for (zyrthid_transform, zyrthid_body, mut target_point) in zyrthid_query.iter_mut() {
        let mut min_dist = f32::MAX;
        let mut target_pos: Option<Vec3> = None;

        for fernworm_transform in fernworm_query.iter() {
            let dist =
                (zyrthid_transform.translation - fernworm_transform.translation).length_squared();
            if dist < min_dist {
                min_dist = dist;
                target_pos = Some(fernworm_transform.translation);
            }
        }

        match target_pos {
            Some(target_pos) => {
                // Algorithm taken from: https://gamedev.stackexchange.com/questions/17313/how-does-one-prevent-homing-missiles-from-orbiting-their-targets
                let v_targ = -zyrthid_body.curr_velocity;
                let s = zyrthid_transform.translation - target_pos;
                let t_estimate = s.length() / (v_targ.length() + f32::EPSILON);

                // Unclear why this constant makes things better, but it
                // prevents oscillating to the left and right when chasing after
                // targets.
                let stability_constant = 0.8;
                let target = target_pos + stability_constant * v_targ * t_estimate
                    - zyrthid_transform.translation;
                target_point.0 = Some(target);
            }
            None => target_point.0 = None,
        }
    }
}

fn eat_berries(
    mut commands: Commands,
    mut game_data: ResMut<SimData>,
    berry_query: Query<(Entity, &mut Transform), (With<Berry>, Without<Fernworm>)>,
    mut fernworm_query: Query<(&mut Transform, &Rotation, &mut Hunger), With<Fernworm>>,
) {
    for (berry_entity, berry_transform) in berry_query.iter() {
        let berry_size =
            berry_transform.scale.truncate() * Vec2::new(BERRY_RENDER_WIDTH, BERRY_RENDER_HEIGHT);

        for (fernworm_transform, rotation, mut hunger) in fernworm_query.iter_mut() {
            // TODO: Extract these parameters to some form of config file/code.
            let mouth_offset = fernworm_transform.scale[1] * FERNWORM_RENDER_HEIGHT / 3.0;
            let mouth_translation = fernworm_transform.translation
                + Vec3::new(
                    -rotation.0.sin() * mouth_offset,
                    rotation.0.cos() * mouth_offset,
                    0.0,
                );
            let mouth_size = fernworm_transform.scale.truncate()
                * Vec2::new(
                    2.0 * FERNWORM_RENDER_WIDTH / 3.0,
                    FERNWORM_RENDER_HEIGHT / 8.0,
                );

            let berry = Aabb2d::new(berry_transform.translation.truncate(), berry_size);
            let mouth = Aabb2d::new(mouth_translation.truncate(), mouth_size);

            if berry.intersects(&mouth) {
                commands.entity(berry_entity).despawn();
                game_data.num_berries -= 1;

                let new_fullness = hunger.curr_fullness + BERRY_FULLNESS_GAIN;
                if new_fullness > hunger.max_fullness {
                    hunger.curr_fullness = hunger.max_fullness;
                } else {
                    hunger.curr_fullness = new_fullness;
                }

                // Break here so that no other unit can eat the same target during the same frame.
                break;
            }
        }
    }
}

fn eat_fernworms(
    mut commands: Commands,
    fernworm_query: Query<(Entity, &mut Transform), (With<Fernworm>, Without<Zyrthid>)>,
    mut zyrthid_query: Query<(&mut Transform, &Rotation, &mut Hunger), With<Zyrthid>>,
) {
    for (fernworm_entity, fernworm_transform) in fernworm_query.iter() {
        // fernworm_RENDER_WIDTH is intentionally used in both dimensions.
        let fernworm_size = fernworm_transform.scale.truncate()
            * Vec2::new(FERNWORM_RENDER_WIDTH / 2.0, FERNWORM_RENDER_WIDTH / 2.0);

        for (zyrthid_transform, rotation, mut hunger) in zyrthid_query.iter_mut() {
            // TODO: Extract these parameters to some form of config file/code.
            let mouth_offset = zyrthid_transform.scale.truncate()[1] * ZYRTHID_RENDER_HEIGHT / 3.0;
            let mouth_translation = zyrthid_transform.translation
                + Vec3::new(
                    -rotation.0.sin() * mouth_offset,
                    rotation.0.cos() * mouth_offset,
                    0.0,
                );
            let mouth_size = zyrthid_transform.scale.truncate() * Vec2::new(3.0, 3.0);

            let fernworm = Aabb2d::new(fernworm_transform.translation.truncate(), fernworm_size);
            let mouth = Aabb2d::new(mouth_translation.truncate(), mouth_size);

            if fernworm.intersects(&mouth) {
                commands.entity(fernworm_entity).despawn();

                let new_fullness = hunger.curr_fullness + FERNWORM_FULLNESS_GAIN;
                if new_fullness > hunger.max_fullness {
                    hunger.curr_fullness = hunger.max_fullness;
                } else {
                    hunger.curr_fullness = new_fullness;
                }

                // Break here so that no other unit can eat the same target during the same frame.
                break;
            }
        }
    }
}

fn repel_bodies(mut body_query: Query<(&mut Transform, &mut MovingBody)>, time: Res<Time>) {
    let mut combinations = body_query.iter_combinations_mut();
    while let Some([mut t1, mut t2]) = combinations.fetch_next() {
        // Bounds before collision force is applied.
        const COLLISION_RADIUS_SQUARED: f32 = (FERNWORM_RENDER_HEIGHT
            + FERNWORM_RENDER_WIDTH / 2.0)
            * (FERNWORM_RENDER_HEIGHT + FERNWORM_RENDER_WIDTH / 2.0);

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

fn hunger_drain(mut query: Query<(&Transform, &mut Hunger)>) {
    for (transform, mut hunger) in query.iter_mut() {
        let dist = hunger.last_sampled_pos.distance(transform.translation);
        hunger.last_sampled_pos = transform.translation;

        hunger.curr_fullness -= dist * hunger.drain_per_unit_traveled;
    }
}

fn kill_starved_units(
    mut commands: Commands,
    query: Query<(Entity, &Hunger, &Transform, &Rotation)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, hunger, transform, rotation) in query.iter() {
        if hunger.curr_fullness <= 0.0 {
            // Spawn sprite of the corpse.
            commands.spawn((
                SimulationComponent,
                FernwormCorpse,
                Rotation(rotation.0),
                Sprite {
                    image: asset_server.load("sprites/fernworm_corpse.png"),
                    custom_size: Some(Vec2::new(FERNWORM_RENDER_WIDTH, FERNWORM_RENDER_HEIGHT)),
                    ..default()
                },
                Transform {
                    translation: transform.translation,
                    rotation: Quat::from_rotation_z(rotation.0),
                    ..default()
                },
            ));

            // Despawn the living sprite.
            commands.entity(entity).despawn();
        }
    }
}

fn exit(query: Query<Entity, With<SimulationComponent>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
