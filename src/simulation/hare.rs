use bevy::prelude::*;

use crate::simulation::*;

const WALK_FRAME_COUNT: usize = 5;

#[derive(Component)]
pub struct Hare;

#[derive(Component)]
pub struct WalkAnimation {
    pub timer: Timer,
}

pub fn use_brain(
    mut hare_query: Query<
        (&Transform, &motion::MovingBody, &mut motion::TargetPoint),
        With<Hare>,
    >,
    berry_query: Query<&Transform, With<berry::Berry>>,
) {
    for (hare_transform, moving_body, mut hare_target_point) in hare_query.iter_mut() {
        let mut min_dist = f32::MAX;
        let mut target_berry_pos: Option<Vec3> = None;

        for berry_transform in berry_query.iter() {
            let dist =
                (berry_transform.translation - hare_transform.translation).length_squared();
            if dist < min_dist {
                min_dist = dist;
                target_berry_pos = Some(berry_transform.translation);
            }
        }

        match target_berry_pos {
            Some(target_pos) => {
                let v_targ = -moving_body.curr_velocity;
                let s = hare_transform.translation - target_pos;
                let t_estimate = s.length() / (v_targ.length() + f32::EPSILON);
                let stability_constant = 0.4;
                let target = target_pos + stability_constant * v_targ * t_estimate
                    - hare_transform.translation;
                hare_target_point.0 = Some(target);
            }
            None => hare_target_point.0 = None,
        }
    }
}

pub fn animate_walk(
    time: Res<Time>,
    mut hare_query: Query<
        (&mut Sprite, &motion::MovingBody, &mut WalkAnimation),
        With<Hare>,
    >,
) {
    for (mut sprite, moving_body, mut animation) in hare_query.iter_mut() {
        let direction_row = direction_row(moving_body.curr_velocity);
        animation.timer.tick(time.delta());

        if moving_body.curr_velocity.length_squared() > 0.01 {
            if let Some(atlas) = sprite.texture_atlas.as_mut() {
                let mut frame = atlas.index % WALK_FRAME_COUNT;
                if animation.timer.just_finished() {
                    frame = (frame + 1) % WALK_FRAME_COUNT;
                }
                atlas.index = direction_row * WALK_FRAME_COUNT + frame;
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
