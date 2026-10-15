use bevy::prelude::*;

use crate::state::{AppState, SimulationSeed};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const DEFAULT_SEED: u64 = 42;

/// Use this component for entities that should be despawned upon state exit
#[derive(Component)]
struct MenuComponent;

#[derive(Component)]
struct SeedText;

pub fn menu_plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Menu), setup)
        .add_systems(Update, seed_input_system.run_if(in_state(AppState::Menu)))
        .add_systems(Update, button_system.run_if(in_state(AppState::Menu)))
        .add_systems(OnExit(AppState::Menu), exit);
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    seed: Res<MenuSeed>,
    mut simulation_seed: ResMut<SimulationSeed>,
    mut state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                simulation_seed.0 = seed.0;
                state.set(AppState::Simulation);
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

#[derive(Resource)]
struct MenuSeed(u64);

fn seed_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut seed: ResMut<MenuSeed>,
    mut seed_text: Query<&mut Text, With<SeedText>>,
) {
    let mut changed = false;

    if keyboard_input.just_pressed(KeyCode::Backspace) {
        seed.0 /= 10;
        changed = true;
    }

    for (key, digit) in [
        (KeyCode::Digit0, 0),
        (KeyCode::Digit1, 1),
        (KeyCode::Digit2, 2),
        (KeyCode::Digit3, 3),
        (KeyCode::Digit4, 4),
        (KeyCode::Digit5, 5),
        (KeyCode::Digit6, 6),
        (KeyCode::Digit7, 7),
        (KeyCode::Digit8, 8),
        (KeyCode::Digit9, 9),
    ] {
        if keyboard_input.just_pressed(key) {
            seed.0 = seed.0.saturating_mul(10).saturating_add(digit);
            changed = true;
        }
    }

    if changed {
        for mut text in &mut seed_text {
            **text = format!("Enter random seed: {}", seed.0);
        }
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(MenuSeed(DEFAULT_SEED));
    commands.spawn((
        Node {
            // center button
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        MenuComponent,
        children![
            (
                Text::new("Enter random seed: 42"),
                SeedText,
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ),
            (
                Button,
                Node {
                    width: Val::Px(150.),
                    height: Val::Px(65.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(NORMAL_BUTTON),
                children![(
                    Text::new("Play"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                )],
            ),
        ],
    ));
}

fn exit(query: Query<Entity, With<MenuComponent>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
