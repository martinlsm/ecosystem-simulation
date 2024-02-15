use bevy::{math::const_vec3, prelude::*, sprite::collide_aabb::collide};
use iyes_loopless::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

use crate::state::AppState;

const NUM_HERBIVORES: usize = 10;
const MAX_BERRIES: u64 = 8;

pub const TIME_STEP_MS: u64 = 30;
pub const TIME_STEP_SEC: f32 = (TIME_STEP_MS as f32) / 1000.0;

pub struct GameData {
    cam: Entity,
    num_berries: u64,
    max_berries: u64,
}

#[derive(Component)]
pub struct MovingBody {
    curr_velocity: Vec3,
    max_speed: f32,
    curr_acceleration: Vec3,
    max_acceleration: f32,
}

#[derive(Component)]
pub struct TargetPoint(Vec3);

#[derive(Component)]
pub struct Rotation(f32);

#[derive(Component)]
pub struct Herbivore;

#[derive(Component)]
pub struct Berry;

pub fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    let cam: Entity = commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .id();

    let mut rng = rand::thread_rng();

    for _ in 0..NUM_HERBIVORES {
        let init_pos = Vec3::new(
            400.0 * rng.gen::<f32>() - 200.0,
            400.0 * rng.gen::<f32>() - 200.0,
            2.0,
        );

        commands
            .spawn()
            .insert(Herbivore)
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: init_pos,
                    scale: Vec3::new(4.0, 4.0, 1.0),
                    ..default()
                },
                texture: asset_server.load("sprites/herbivore.png"),
                ..default()
            })
            .insert(MovingBody {
                curr_velocity: Vec3::ZERO,
                max_speed: 200.0,
                curr_acceleration: Vec3::ZERO,
                max_acceleration: 1000.0,
            })
            .insert(TargetPoint(const_vec3!([0.0, 0.0, 0.0])))
            .insert(Rotation(0.0));
    }

    commands.insert_resource(GameData {
        cam: cam,
        num_berries: 0,
        max_berries: MAX_BERRIES,
    });
}

pub fn cleanup_game(mut commands: Commands, game_data: Res<GameData>, entities: Query<Entity>) {
    commands.entity(game_data.cam).despawn_recursive();

    for e in entities.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn apply_velocity(mut query: Query<(&mut Rotation, &mut Transform, &MovingBody)>) {
    for (mut rotation, mut transform, moving_body) in query.iter_mut() {
        // Update position.
        transform.translation = transform.translation + moving_body.curr_velocity * TIME_STEP_SEC;

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

pub fn handle_input(mut commands: Commands, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(AppState::Menu));

        return;
    }
}

pub fn update_velocity(mut query: Query<(&mut MovingBody, &TargetPoint)>) {
    for (mut moving_body, target_point) in query.iter_mut() {
        moving_body.curr_acceleration = (target_point.0.normalize_or_zero()
            * moving_body.max_acceleration)
            .clamp_length_max(TIME_STEP_SEC * moving_body.max_acceleration);
        moving_body.curr_velocity = (moving_body.curr_velocity + moving_body.curr_acceleration)
            .clamp_length_max(moving_body.max_speed);
    }
}

pub fn eat_berries(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
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

            let collision = collide(
                mouth_translation,
                mouth_size,
                berry_transform.translation,
                berry_size,
            );

            if let Some(_collision) = collision {
                commands.entity(berry_entity).despawn();
                game_data.num_berries -= 1;

                // Break here so that no other herbivore can eat the same berry during the same frame.
                break;
            }
        }
    }
}

pub fn spawn_berries(
    mut game_data: ResMut<GameData>,
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

        commands.spawn().insert(Berry).insert_bundle(SpriteBundle {
            transform: Transform {
                translation: init_pos_berry,
                scale: Vec3::new(4.0, 4.0, 1.0),
                ..default()
            },
            texture: asset_server.load("sprites/berry.png"),
            ..default()
        });
    }

    game_data.num_berries = game_data.max_berries;
}

pub fn use_brain(
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