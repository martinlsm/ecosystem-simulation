use bevy::prelude::*;

use crate::simulation::*;

#[derive(Component)]
pub struct Fernworm;

#[derive(Component)]
pub struct Corpse;

pub fn use_brain(
    mut fernworm_query: Query<
        (&Transform, &core::MovingBody, &mut core::TargetPoint),
        With<Fernworm>,
    >,
    berry_query: Query<&Transform, With<berry::Berry>>,
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

pub fn eat_berries(
    mut commands: Commands,
    mut game_data: ResMut<SimData>,
    berry_query: Query<(Entity, &mut Transform), (With<berry::Berry>, Without<Fernworm>)>,
    mut fernworm_query: Query<
        (&mut Transform, &core::Rotation, &mut hunger::Hunger),
        With<Fernworm>,
    >,
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
