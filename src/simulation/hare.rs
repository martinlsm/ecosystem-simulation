use bevy::prelude::*;

use crate::simulation::*;

const WALK_FRAME_COUNT: usize = 5;
const IDLE_FRAME_COUNT: usize = 4;

#[derive(Component)]
pub struct Hare;

#[derive(Component)]
pub struct WalkAnimation {
    pub timer: Timer,
    pub direction_row: usize,
    pub frame: usize,
    pub idle_image: Handle<Image>,
    pub idle_layout: Handle<TextureAtlasLayout>,
    pub walk_image: Handle<Image>,
    pub walk_layout: Handle<TextureAtlasLayout>,
}

pub fn use_brain(
    mut hare_query: Query<
        (&Transform, &motion::MovingBody, &mut motion::TargetPoint),
        With<Hare>,
    >,
    berry_query: Query<&Transform, With<berry::Berry>>,
    navigation_grid: Res<pathfinding::NavigationGrid>,
) {
    for (hare_transform, _moving_body, mut hare_target_point) in hare_query.iter_mut() {
        let hare_cell = pathfinding::NavigationGrid::world_to_grid(hare_transform.translation);
        let mut nearest_berry: Option<Vec3> = None;
        let mut nearest_distance = f32::MAX;

        for berry_transform in berry_query.iter() {
            let distance = berry_transform
                .translation
                .distance_squared(hare_transform.translation);
            if distance < nearest_distance {
                nearest_distance = distance;
                nearest_berry = Some(berry_transform.translation);
            }
        }

        hare_target_point.0 = nearest_berry
            .and_then(|berry| {
                let berry_cell = pathfinding::NavigationGrid::world_to_grid(berry);
                navigation_grid.find_path(
                    hare_cell,
                    berry_cell,
                )
                .map(|mut path| (berry, path.pop_front(), path.is_empty()))
            })
            .map(|(berry, next_cell, is_final_step)| {
                if is_final_step {
                    berry - hare_transform.translation
                } else {
                    pathfinding::NavigationGrid::grid_to_world(next_cell.unwrap())
                        - hare_transform.translation
                }
            });
    }
}

pub fn animate(
    time: Res<Time>,
    mut hare_query: Query<
        (&mut Sprite, &motion::MovingBody, &mut WalkAnimation),
        With<Hare>,
    >,
) {
    for (mut sprite, moving_body, mut animation) in hare_query.iter_mut() {
        if moving_body.curr_velocity.length_squared() > 0.01 {
            animation.direction_row = direction_row(moving_body.curr_velocity);
        }
        animation.timer.tick(time.delta());

        let is_moving = moving_body.curr_velocity.length_squared() > 0.01;
        if is_moving {
            if sprite.image != animation.walk_image {
                sprite.image = animation.walk_image.clone();
                animation.frame = 0;
            }
        } else if sprite.image != animation.idle_image {
            sprite.image = animation.idle_image.clone();
            animation.frame = 0;
        }

        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            if is_moving {
                atlas.layout = animation.walk_layout.clone();
                if animation.timer.just_finished() {
                    animation.frame = (animation.frame + 1) % WALK_FRAME_COUNT;
                }
                atlas.index = animation.direction_row * WALK_FRAME_COUNT + animation.frame;
            } else {
                atlas.layout = animation.idle_layout.clone();
                if animation.timer.just_finished() {
                    animation.frame = (animation.frame + 1) % IDLE_FRAME_COUNT;
                }
                atlas.index = animation.direction_row * IDLE_FRAME_COUNT + animation.frame;
            }
        }
    }
}

fn direction_row(velocity: Vec3) -> usize {
    if velocity.y.abs() >= velocity.x.abs() {
        if velocity.y >= 0.0 { 1 } else { 0 }
    } else if velocity.x < 0.0 {
        2
    } else {
        3
    }
}

pub fn eat_berries(
    mut commands: Commands,
    mut game_data: ResMut<SimData>,
    berry_query: Query<(Entity, &Transform), With<berry::Berry>>,
    hare_query: Query<&Transform, With<Hare>>,
) {
    for (berry_entity, berry_transform) in berry_query.iter() {
        let berry = Aabb2d::new(
            berry_transform.translation.truncate(),
            berry_transform.scale.truncate() * Vec2::new(BERRY_RENDER_WIDTH, BERRY_RENDER_HEIGHT),
        );

        if hare_query.iter().any(|hare_transform| {
            let hare = Aabb2d::new(
                hare_transform.translation.truncate(),
                Vec2::new(
                    HARE_SPRITE_WIDTH as f32 / 10.0,
                    HARE_SPRITE_HEIGHT as f32 / 10.0,
                ),
            );
            berry.intersects(&hare)
        }) {
            commands.entity(berry_entity).despawn();
            game_data.num_berries -= 1;
        }
    }
}
