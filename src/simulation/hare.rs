use bevy::prelude::*;
use rand::{rngs::StdRng, Rng};
use std::f32::consts::TAU;

use crate::simulation::*;

const WALK_FRAME_COUNT: usize = 5;
const IDLE_FRAME_COUNT: usize = 4;
const BERRY_SEEK_RADIUS: f32 = TILE_WIDTH * 2.0;

#[derive(Component)]
pub struct Hare;

#[derive(Component)]
pub struct HareAnimation {
    pub timer: Timer,
    pub direction_row: usize,
    pub frame: usize,
    pub idle_image: Handle<Image>,
    pub idle_layout: Handle<TextureAtlasLayout>,
    pub walk_image: Handle<Image>,
    pub walk_layout: Handle<TextureAtlasLayout>,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum HareState {
    RoamingIdle,
    RoamingWalking,
    SeekingFood,
    // Eating,
}

#[derive(Component)]
pub struct RoamingBehavior {
    pub timer: Timer,
    pub direction: Vec3,
    pub destination: Option<pathfinding::GridPosition>,
}

pub fn spawn(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    rng: &mut StdRng,
    navigation_grid: &pathfinding::NavigationGrid,
) {
    let hare_atlas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(HARE_SPRITE_WIDTH, HARE_SPRITE_HEIGHT),
        5,
        4,
        None,
        None,
    ));
    let hare_idle_atlas_layout = texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(HARE_SPRITE_WIDTH, HARE_SPRITE_HEIGHT),
        4,
        4,
        None,
        None,
    ));
    let hare_idle_image =
        asset_server.load("sprites/animals/PNG/Without_shadow/Hare/Hare_Idle.png");
    let hare_walk_image =
        asset_server.load("sprites/animals/PNG/Without_shadow/Hare/Hare_Walk.png");

    for _ in 0..NUM_HARES {
        let init_pos = loop {
            let position = Vec3::new(
                rng.gen_range(PLAYABLE_AREA_X0..PLAYABLE_AREA_X1),
                rng.gen_range(PLAYABLE_AREA_Y0..PLAYABLE_AREA_Y1),
                2.0,
            );
            if !navigation_grid.is_blocked(position) {
                break position;
            }
        };

        commands.spawn((
            SimulationComponent,
            Hare,
            HareState::RoamingWalking,
            RoamingBehavior {
                timer: Timer::from_seconds(rng.gen_range(1.0..3.0), TimerMode::Once),
                direction: random_direction(rng),
                destination: None,
            },
            HareAnimation {
                timer: Timer::from_seconds(0.12, TimerMode::Repeating),
                direction_row: 0,
                frame: 0,
                idle_image: hare_idle_image.clone(),
                idle_layout: hare_idle_atlas_layout.clone(),
                walk_image: hare_walk_image.clone(),
                walk_layout: hare_atlas_layout.clone(),
            },
            Sprite {
                image: hare_idle_image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: hare_idle_atlas_layout.clone(),
                    index: 0,
                }),
                ..default()
            },
            Transform::from_translation(init_pos),
            motion::MovingBody {
                curr_velocity: Vec3::ZERO,
                max_speed: HARE_MAX_SPEED,
                curr_acceleration: Vec3::ZERO,
                max_acceleration: 1000.0,
            },
            motion::TargetPoint(None),
        ));
    }
}

pub fn decision_system(
    time: Res<Time>,
    mut hare_query: Query<
        (
            &Transform,
            &mut motion::TargetPoint,
            &mut HareState,
            &mut RoamingBehavior,
        ),
        With<Hare>,
    >,
    berry_query: Query<&Transform, With<berry::Berry>>,
    navigation_grid: Res<pathfinding::NavigationGrid>,
) {
    for (hare_transform, mut hare_target_point, mut hare_state, mut roaming) in
        hare_query.iter_mut()
    {
        let hare_cell = pathfinding::NavigationGrid::world_to_grid(hare_transform.translation);
        let mut nearest_berry: Option<Vec3> = None;
        let mut nearest_distance = f32::MAX;

        for berry_transform in berry_query.iter() {
            let distance = berry_transform
                .translation
                .distance_squared(hare_transform.translation);
            if distance <= BERRY_SEEK_RADIUS * BERRY_SEEK_RADIUS && distance < nearest_distance {
                nearest_distance = distance;
                nearest_berry = Some(berry_transform.translation);
            }
        }

        let target_point = nearest_berry
            .and_then(|berry| {
                let berry_cell = pathfinding::NavigationGrid::world_to_grid(berry);
                navigation_grid
                    .find_path(hare_cell, berry_cell)
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

        if let Some(target_point) = target_point {
            *hare_state = HareState::SeekingFood;
            hare_target_point.0 = Some(target_point);
            continue;
        }

        hare_target_point.0 = match *hare_state {
            HareState::SeekingFood => {
                enter_state_roaming_idle(&mut hare_state, &mut roaming);
                None
            }
            HareState::RoamingIdle => {
                roaming.timer.tick(time.delta());
                if roaming.timer.just_finished() {
                    enter_state_roaming_walking(
                        &mut hare_state,
                        &mut roaming,
                        hare_cell,
                        hare_transform.translation,
                        &navigation_grid,
                    )
                } else {
                    None
                }
            }
            HareState::RoamingWalking => {
                roaming.timer.tick(time.delta());
                if roaming.timer.just_finished() || roaming.destination == Some(hare_cell) {
                    enter_state_roaming_idle(&mut hare_state, &mut roaming);
                    None
                } else {
                    roaming_target_point(
                        hare_cell,
                        hare_transform.translation,
                        &mut roaming,
                        &navigation_grid,
                    )
                }
            }
        };
    }
}

fn enter_state_roaming_idle(hare_state: &mut HareState, roaming: &mut RoamingBehavior) {
    let mut rng = rand::thread_rng();
    *hare_state = HareState::RoamingIdle;
    roaming.timer = Timer::from_seconds(rng.gen_range(2.0..5.0), TimerMode::Once);
    roaming.destination = None;
}

fn enter_state_roaming_walking(
    hare_state: &mut HareState,
    roaming: &mut RoamingBehavior,
    hare_cell: pathfinding::GridPosition,
    hare_position: Vec3,
    navigation_grid: &pathfinding::NavigationGrid,
) -> Option<Vec3> {
    let mut rng = rand::thread_rng();
    *hare_state = HareState::RoamingWalking;
    roaming.timer = Timer::from_seconds(rng.gen_range(1.0..3.0), TimerMode::Once);
    roaming.destination = None;
    roaming_target_point(hare_cell, hare_position, roaming, navigation_grid)
}

fn roaming_target_point(
    hare_cell: pathfinding::GridPosition,
    hare_position: Vec3,
    roaming: &mut RoamingBehavior,
    navigation_grid: &pathfinding::NavigationGrid,
) -> Option<Vec3> {
    let mut rng = rand::thread_rng();

    if roaming.destination.is_none() {
        let mut reachable_cells = navigation_grid.reachable_cells(hare_cell, 4);
        reachable_cells.retain(|&cell| cell != hare_cell);
        roaming.destination = if !reachable_cells.is_empty() {
            Some(reachable_cells[rng.gen_range(0..reachable_cells.len())])
        } else {
            None
        };
    }

    let destination = roaming.destination?;
    let mut path = match navigation_grid.find_path(hare_cell, destination) {
        Some(path) => path,
        None => {
            roaming.destination = None;
            return None;
        }
    };

    let next_cell = path.pop_front()?;
    let target = pathfinding::NavigationGrid::grid_to_world(next_cell) - hare_position;
    let target = target.truncate().extend(0.0);
    roaming.direction = target;
    Some(target)
}

pub fn animate(
    time: Res<Time>,
    mut hare_query: Query<
        (
            &mut Sprite,
            &motion::TargetPoint,
            &HareState,
            &RoamingBehavior,
            &mut HareAnimation,
        ),
        With<Hare>,
    >,
) {
    for (mut sprite, target_point, hare_state, roaming, mut animation) in hare_query.iter_mut() {
        animation.timer.tick(time.delta());

        let (image, layout, frame_count, direction) = match *hare_state {
            HareState::SeekingFood => (
                animation.walk_image.clone(),
                animation.walk_layout.clone(),
                WALK_FRAME_COUNT,
                Some(target_point.0.unwrap_or(Vec3::X)),
            ),
            HareState::RoamingWalking => (
                animation.walk_image.clone(),
                animation.walk_layout.clone(),
                WALK_FRAME_COUNT,
                Some(roaming.direction),
            ),
            HareState::RoamingIdle => (
                animation.idle_image.clone(),
                animation.idle_layout.clone(),
                IDLE_FRAME_COUNT,
                None,
            ),
        };

        if let Some(direction) = direction {
            animation.direction_row = animation_direction_row(direction);
        }
        if sprite.image != image {
            sprite.image = image;
            animation.frame = 0;
        }

        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.layout = layout;
            if animation.timer.just_finished() {
                animation.frame = (animation.frame + 1) % frame_count;
            }
            atlas.index = animation.direction_row * frame_count + animation.frame;
        }
    }
}

fn random_direction(rng: &mut impl Rng) -> Vec3 {
    let angle = rng.gen_range(0.0..TAU);
    Vec3::new(angle.cos(), angle.sin(), 0.0)
}

fn animation_direction_row(velocity: Vec3) -> usize {
    if velocity.y.abs() >= velocity.x.abs() {
        if velocity.y >= 0.0 {
            1
        } else {
            0
        }
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
