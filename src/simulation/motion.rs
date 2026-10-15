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

pub fn apply_velocity(
    mut query: Query<(&mut Transform, &MovingBody)>,
    rock_query: Query<&Transform, (With<MapRock>, Without<MovingBody>)>,
    time: Res<Time>,
) {
    for (mut transform, moving_body) in query.iter_mut() {
        let previous_position = transform.translation;
        let mut next_position = previous_position + moving_body.curr_velocity * time.delta_secs();
        next_position.x = next_position.x.clamp(PLAYABLE_AREA_X0, PLAYABLE_AREA_X1);
        next_position.y = next_position.y.clamp(PLAYABLE_AREA_Y0, PLAYABLE_AREA_Y1);

        let entered_rock = rock_query.iter().any(|rock| {
            let distance = next_position - rock.translation;
            distance.x.abs() < TILE_WIDTH / 2.0 && distance.y.abs() < TILE_HEIGHT / 2.0
        });
        transform.translation = if entered_rock {
            previous_position
        } else {
            next_position
        };
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

pub fn repel_bodies(
    mut body_query: Query<(&mut Transform, &mut MovingBody)>,
    time: Res<Time>,
) {
    let mut combinations = body_query.iter_combinations_mut();
    while let Some([mut t1, mut t2]) = combinations.fetch_next() {
        // Bounds before collision force is applied.
        const COLLISION_RADIUS_SQUARED: f32 = (FERNWORM_RENDER_HEIGHT
            + FERNWORM_RENDER_WIDTH / 2.0)
            * (FERNWORM_RENDER_HEIGHT + FERNWORM_RENDER_WIDTH / 2.0);

        // Strength of the collision force.
        const FORCE_CONSTANT: f32 = 50000.0;

        let p1: Vec3 = t1.0.as_mut().translation;
        let p2: Vec3 = t2.0.as_mut().translation;
        let squared_dist = p1.distance_squared(p2);

        let force = if squared_dist < COLLISION_RADIUS_SQUARED {
            FORCE_CONSTANT / (squared_dist + f32::EPSILON)
        } else {
            0.0
        };

        let push_direction = (p1 - p2).normalize_or_zero() * force * time.delta_secs();

        let moving_body1 = t1.1.as_mut();
        moving_body1.curr_velocity += push_direction;

        let moving_body2 = t2.1.as_mut();
        moving_body2.curr_velocity -= push_direction;
    }
}
