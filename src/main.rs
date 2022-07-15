use bevy::{math::const_vec3, prelude::*, sprite::collide_aabb::collide};
use iyes_loopless::prelude::*;
use rand::Rng;
use std::f32::consts::PI;
use std::time::Duration;

const TIME_STEP_MS: u64 = 30;
const TIME_STEP_SEC: f32 = (TIME_STEP_MS as f32) / 1000.0;

fn main() {
    let mut fixed_update = SystemStage::parallel();
    fixed_update.add_system(apply_velocity.run_in_state(AppState::InGame));
    fixed_update.add_system(update_velocity.run_in_state(AppState::InGame));
    fixed_update.add_system(handle_input.run_in_state(AppState::InGame));
    fixed_update.add_system(eat_berries.run_in_state(AppState::InGame));
    fixed_update.add_system(spawn_berries.run_in_state(AppState::InGame));

    App::new()
        .add_plugins(DefaultPlugins)
        .add_loopless_state(AppState::Menu)
        .add_stage_before(
            CoreStage::Update,
            "FixedUpdate",
            FixedTimestepStage::from_stage(Duration::from_millis(TIME_STEP_MS), fixed_update),
        )
        .add_enter_system(AppState::Menu, setup_menu)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(AppState::Menu)
                .with_system(menu)
                .into(),
        )
        .add_exit_system(AppState::Menu, cleanup_menu)
        .add_enter_system(AppState::InGame, setup_game)
        .add_exit_system(AppState::InGame, cleanup_game)
        .run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Menu,
    InGame,
}

struct MenuData {
    button_entity: Entity,
}

struct GameData {
    cam: Entity,
    num_berries: u64,
    max_berries: u64,
}

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct TargetDir(Vec3);

#[derive(Component)]
struct Rotation(f32);

#[derive(Component)]
struct Herbivore;

#[derive(Component)]
struct Berry;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());

    let button_entity = commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                margin: Rect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Play",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..default()
            });
        })
        .id();

    commands.insert_resource(MenuData { button_entity });
}

fn menu(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                commands.insert_resource(NextState(AppState::InGame));
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.button_entity).despawn_recursive();
}

const NUM_ACTORS: usize = 1;

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    let cam: Entity = commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .id();

    let mut rng = rand::thread_rng();

    for _ in 0..NUM_ACTORS {
        let init_pos = Vec3::new(
            400.0 * rng.gen::<f32>() - 200.0,
            400.0 * rng.gen::<f32>() - 200.0,
            1.0,
        );

        commands
            .spawn()
            .insert(Herbivore)
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: init_pos,
                    scale: Vec3::new(4.0, 4.0, 1.0),
                    ..default()
                },
                texture: asset_server.load("sprites/herbivore.png"),
                ..default()
            })
            .insert(Velocity(const_vec3!([0.0, 0.0, 0.0])))
            .insert(TargetDir(const_vec3!([0.0, 0.0, 0.0])))
            .insert(Rotation(0.0));
    }

    commands.insert_resource(GameData {
        cam: cam,
        num_berries: 0,
        max_berries: 50,
    });
}

fn cleanup_game(mut commands: Commands, game_data: Res<GameData>, entities: Query<Entity>) {
    commands.entity(game_data.cam).despawn_recursive();

    for e in entities.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn apply_velocity(mut query: Query<(&mut Rotation, &mut Transform, &Velocity)>) {
    for (mut rotation, mut transform, velocity) in query.iter_mut() {
        // Update position.
        transform.translation = transform.translation + velocity.0 * TIME_STEP_SEC;

        // Update sprite rotation.
        if velocity.0.length() > 0.001 {
            let mut angle = (velocity.0.y / velocity.0.x).atan() - PI / 2.0;
            if velocity.0.x < 0.0 {
                angle += PI;
            }

            rotation.0 = angle;
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

fn handle_input(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut TargetDir, With<Herbivore>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        commands.insert_resource(NextState(AppState::Menu));

        return;
    }

    let mut new_direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::Left) {
        new_direction.x = -1.0;
    } else if keyboard_input.pressed(KeyCode::Right) {
        new_direction.x = 1.0;
    }

    if keyboard_input.pressed(KeyCode::Up) {
        new_direction.y = 1.0;
    } else if keyboard_input.pressed(KeyCode::Down) {
        new_direction.y = -1.0;
    }

    new_direction = new_direction.normalize_or_zero() * 200.0;

    for mut target_dir in query.iter_mut() {
        target_dir.0 = new_direction;
    }
}

fn update_velocity(mut query: Query<(&mut Velocity, &TargetDir)>) {
    const UPDATE_STEP: f32 = 0.1;

    for (mut velocity, target_dir) in query.iter_mut() {
        velocity.0 = (1.0 - UPDATE_STEP) * velocity.0 + UPDATE_STEP * target_dir.0;
    }
}

fn eat_berries(
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    berry_query: Query<(Entity, &mut Transform), (With<Berry>, Without<Herbivore>)>,
    herbivore_query: Query<(&mut Transform, &Rotation), (With<Herbivore>, Without<Berry>)>,
) {
    for (berry_entity, berry_transform) in berry_query.iter() {
        let berry_size = berry_transform.scale.truncate() * Vec2::new(2.0, 2.0);

        for (herbivore_transform, rotation) in herbivore_query.iter() {
            let mouth_offset = herbivore_transform.scale.truncate() * 5.0;
            let mouth_translation = herbivore_transform.translation + Vec3::new(-rotation.0.sin() * mouth_offset.x, rotation.0.cos() * mouth_offset.y, 0.0);

            let mouth_size = herbivore_transform.scale.truncate() * Vec2::new(4.0, 2.0);

            let collision = collide(
                mouth_translation,
                mouth_size,
                berry_transform.translation,
                berry_size,
            );

            if let Some(_collision) = collision {
                commands.entity(berry_entity).despawn();
                game_data.num_berries -= 1;
            }
        }
    }
}

fn spawn_berries(
    mut game_data: ResMut<GameData>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::thread_rng();

    for _ in game_data.num_berries..game_data.max_berries {
        let init_pos_berry = Vec3::new(
            400.0 * rng.gen::<f32>() - 200.0,
            400.0 * rng.gen::<f32>() - 200.0,
            2.0,
        );

        commands.spawn().insert(Berry).insert_bundle(SpriteBundle {
            transform: Transform {
                translation: init_pos_berry,
                scale: Vec3::new(4.0, 4.0, 1.0),
                ..default()
            },
            texture: asset_server.load("sprites/berry.png"),
            ..default()
        });
    }

    game_data.num_berries = game_data.max_berries;
}
