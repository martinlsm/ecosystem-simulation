pub mod fernworm;
pub mod zyrthid;

use crate::simulation::*;
use bevy::prelude::*;

#[derive(Component, Copy, Clone)]
pub enum UnitType {
    Fernworm,
    Zyrthid,
}

pub struct CorpseData {
    pub unit: UnitType,
    pub translation: Vec3,
    pub rotation: f32,
}

#[derive(Event)]
pub struct DeathEvent {
    pub entity: Entity,
    pub corpse: Option<unit::CorpseData>,
}

pub fn kill_units(
    mut commands: Commands,
    mut events: EventReader<DeathEvent>,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        // Despawn the living sprite.
        commands.entity(event.entity).despawn();

        if let Some(corpse) = &event.corpse {
            let custom_size = match corpse.unit {
                UnitType::Fernworm => Vec2::new(FERNWORM_RENDER_WIDTH, FERNWORM_RENDER_HEIGHT),
                UnitType::Zyrthid => Vec2::new(ZYRTHID_RENDER_WIDTH, ZYRTHID_RENDER_HEIGHT),
            };

            commands.spawn((
                SimulationComponent,
                Sprite {
                    image: match corpse.unit {
                        unit::UnitType::Fernworm => {
                            asset_server.load("sprites/fernworm_corpse.png")
                        }
                        unit::UnitType::Zyrthid => asset_server.load("sprites/zyrthid_corpse.png"),
                    },
                    custom_size: Some(custom_size),
                    ..default()
                },
                Transform {
                    translation: corpse.translation,
                    rotation: Quat::from_rotation_z(corpse.rotation),
                    ..default()
                },
            ));
        }
    }
}
