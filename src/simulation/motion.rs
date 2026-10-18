use bevy::prelude::*;

use crate::simulation::*;

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
    time: Res<Time>,
    navigation_grid: Res<pathfinding::NavigationGrid>,
) {
    for (mut transform, moving_body) in query.iter_mut() {
        let movement = moving_body.curr_velocity * time.delta_secs();
        let mut next_position = transform.translation;

        let horizontal_position = Vec3::new(
            (next_position.x + movement.x).clamp(PLAYABLE_AREA_X0, PLAYABLE_AREA_X1),
            next_position.y,
            next_position.z,
        );
        if !navigation_grid.is_blocked(horizontal_position) {
            next_position.x = horizontal_position.x;
        }

        let vertical_position = Vec3::new(
            next_position.x,
            (next_position.y + movement.y).clamp(PLAYABLE_AREA_Y0, PLAYABLE_AREA_Y1),
            next_position.z,
        );
        if !navigation_grid.is_blocked(vertical_position) {
            next_position.y = vertical_position.y;
        }

        transform.translation = next_position;
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
            moving_body.curr_acceleration = Vec3::ZERO;
            moving_body.curr_velocity = Vec3::ZERO;
        }
    }
}

pub fn repel_bodies(
    mut body_query: Query<(&mut Transform, &mut MovingBody), Without<MapRock>>,
    time: Res<Time>,
) {
    let mut combinations = body_query.iter_combinations_mut();
    while let Some([mut t1, mut t2]) = combinations.fetch_next() {
        // Bounds before collision force is applied.
        const COLLISION_RADIUS_SQUARED: f32 = HARE_SPRITE_HEIGHT as f32 * HARE_SPRITE_HEIGHT as f32;

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
