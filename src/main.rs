use bevy::{
    core::FixedTimestep,
    prelude::*,
    math::const_vec3,
};

use std::f32::consts::PI;

use rand::Rng;

const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state(AppState::Menu)
        .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(setup_menu))
        .add_system_set(SystemSet::on_update(AppState::Menu).with_system(menu))
        .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(cleanup_menu))
        .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup_game))
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(handle_input)
                .with_system(update_velocity)
                .with_system(apply_velocity)
        )
        .add_system_set(SystemSet::on_exit(AppState::InGame).with_system(cleanup_game))
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
}

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct TargetDir(Vec3);

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

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
    mut state: ResMut<State<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                state.set(AppState::InGame).unwrap();
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

const NUM_ACTORS: usize = 10;

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    let cam: Entity = commands.spawn_bundle(OrthographicCameraBundle::new_2d()).id();

    let mut rng = rand::thread_rng();

    for _ in 0..NUM_ACTORS {
        let init_pos = Vec3::new(400.0 * rng.gen::<f32>() - 200.0, 400.0 * rng.gen::<f32>() - 200.0, 0.0);

        commands
            .spawn()
            .insert_bundle(SpriteBundle {
                transform: Transform {
                    translation: init_pos,
                    scale: Vec3::new(2.0, 2.0, 1.0),
                    ..default()
                },
                texture: asset_server.load("branding/herbivore2.png"),
                ..default()
            })
            .insert(Velocity(const_vec3!([0.0, 0.0, 0.0])))
            .insert(TargetDir(const_vec3!([0.0, 0.0, 0.0])));

        commands.insert_resource(GameData { cam });
    }
}

fn cleanup_game(mut commands: Commands, game_data: Res<GameData>, entities: Query<Entity>) {
    commands.entity(game_data.cam).despawn_recursive();

    for e in entities.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        // Update position.
        transform.translation = transform.translation + velocity.0 * TIME_STEP;

        // Update sprite rotation.
        if velocity.0.length() > 0.001 {
            let mut angle = (velocity.0.y / velocity.0.x).atan() - PI / 2.0;
            if velocity.0.x < 0.0 {
                angle += PI;
            }
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

fn handle_input(mut state: ResMut<State<AppState>>,
    keyboard_input: Res<Input<KeyCode>>, mut query: Query<&mut TargetDir>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        state.set(AppState::Menu).unwrap();
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

    new_direction = new_direction.normalize_or_zero() * 100.0;

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