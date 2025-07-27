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
    query: Query<(
        Entity,
        &unit::UnitType,
        &Hunger,
        &Transform,
        &motion::Rotation,
    )>,
    mut events: EventWriter<unit::DeathEvent>,
) {
    for (entity, unit, hunger, transform, rotation) in query.iter() {
        if hunger.curr_fullness <= 0.0 {
            events.write(unit::DeathEvent {
                entity: entity,
                corpse: Some(unit::CorpseData {
                    unit: *unit,
                    translation: transform.translation,
                    rotation: rotation.0,
                }),
            });
        }
    }
}
