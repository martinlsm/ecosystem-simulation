use crate::state::AppState;

use bevy::prelude::*;
use rand::Rng;

const NUM_HERBIVORES: usize = 1;

#[derive(Component)]
struct MovingBody {
    curr_velocity: Vec3,
    max_speed: f32,
    curr_acceleration: Vec3,
    max_acceleration: f32,
}

#[derive(Component)]
struct SimulationComponent;

#[derive(Component)]
struct Rotation(f32);

#[derive(Component)]
struct Herbivore;

#[derive(Component)]
struct TargetPoint(Vec3);

pub fn simulation_plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Simulation), setup)
        .add_systems(OnExit(AppState::Simulation), exit)
     .add_systems(Update, handle_input.run_if(in_state(AppState::Simulation)));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::thread_rng();

    for _ in 0..NUM_HERBIVORES {
        let init_pos = Vec3::new(
            400.0 * rng.gen::<f32>() - 200.0,
            400.0 * rng.gen::<f32>() - 200.0,
            2.0,
        );

        commands.spawn((
            SimulationComponent,
            Herbivore,
            Rotation(0.0),
            SpriteBundle {
                transform: Transform {
                    translation: init_pos,
                    scale: Vec3::new(4.0, 4.0, 1.0),
                    ..default()
                },
                texture: asset_server.load("sprites/herbivore.png"),
                ..default()
            },
            MovingBody {
                curr_velocity: Vec3::ZERO,
                max_speed: 200.0,
                curr_acceleration: Vec3::ZERO,
                max_acceleration: 1000.0,
            },
            TargetPoint(bevy::math::vec3(0.0, 0.0, 0.0)),
        ));
    }
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        state.set(AppState::Menu);
    }
}

fn exit(query: Query<Entity, With<SimulationComponent>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
