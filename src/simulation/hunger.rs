use bevy::prelude::*;

use crate::simulation::*;

#[derive(Component)]
pub struct Hunger {
    pub curr_fullness: f32,
    pub max_fullness: f32,
    pub drain_per_unit_traveled: f32,
    pub last_sampled_pos: Vec3,
}

pub fn hunger_drain(mut query: Query<(&Transform, &mut Hunger)>) {
    for (transform, mut hunger) in query.iter_mut() {
        let dist = hunger.last_sampled_pos.distance(transform.translation);
        hunger.last_sampled_pos = transform.translation;

        hunger.curr_fullness -= dist * hunger.drain_per_unit_traveled;
    }
}

pub fn kill_starved_units(
    mut commands: Commands,
    query: Query<(Entity, &Hunger, &Transform, &core::Rotation)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, hunger, transform, rotation) in query.iter() {
        if hunger.curr_fullness <= 0.0 {
            // Spawn sprite of the corpse.
            commands.spawn((
                SimulationComponent,
                fernworm::Corpse,
                core::Rotation(rotation.0),
                Sprite {
                    image: asset_server.load("sprites/fernworm_corpse.png"), // TODO: Should not be hardcoded
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
