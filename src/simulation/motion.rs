use bevy::prelude::*;

use crate::simulation::*;

#[derive(Component)]
pub struct Rotation(pub f32);

#[derive(Component)]
pub struct TargetPoint(pub Option<Vec3>);

#[derive(Component)]
pub struct MovingBody {
    pub curr_velocity: Vec3,
    pub max_speed: f32,
    pub curr_acceleration: Vec3,
    pub max_acceleration: f32,
}

pub fn apply_velocity(mut query: Query<(&mut Transform, &MovingBody)>, time: Res<Time>) {
    for (mut transform, moving_body) in query.iter_mut() {
        // Update position.
        transform.translation =
            transform.translation + moving_body.curr_velocity * time.delta_secs();
    }
}

pub fn apply_rotation(mut query: Query<(&mut Rotation, &mut Transform, &MovingBody)>) {
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

pub fn update_velocity(mut query: Query<(&mut MovingBody, &TargetPoint)>, time: Res<Time>) {
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

pub fn repel_bodies(mut body_query: Query<(&mut Transform, &mut MovingBody)>, time: Res<Time>) {
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
