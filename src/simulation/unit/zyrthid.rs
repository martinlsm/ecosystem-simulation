use bevy::prelude::*;

use crate::simulation::*;

#[derive(Component)]
pub struct Zyrthid;

pub fn use_brain(
    mut zyrthid_query: Query<
        (&Transform, &motion::MovingBody, &mut motion::TargetPoint),
        With<Zyrthid>,
    >,
    fernworm_query: Query<&Transform, With<fernworm::Fernworm>>,
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

pub fn eat_fernworms(
    fernworm_query: Query<(Entity, &mut Transform), (With<fernworm::Fernworm>, Without<Zyrthid>)>,
    mut zyrthid_query: Query<
        (&mut Transform, &motion::Rotation, &mut hunger::Hunger),
        With<Zyrthid>,
    >,
    mut event: EventWriter<unit::DeathEvent>,
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
                event.write(unit::DeathEvent {
                    entity: fernworm_entity,
                    corpse: None,
                });

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
