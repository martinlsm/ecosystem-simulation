use bevy::prelude::*;

use crate::state::AppState;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);

/// Use this component for entities that should be despawned upon state exit
#[derive(Component)]
struct MenuComponent;

pub fn menu_plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Menu), setup)
        .add_systems(Update, button_system.run_if(in_state(AppState::Menu)))
        .add_systems(OnExit(AppState::Menu), exit);
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
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

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                // center button
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            MenuComponent,
            children![(
                Button,
                Node {
                    width: Val::Px(150.),
                    height: Val::Px(65.),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
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
            )],
        ));
}

fn exit(query: Query<Entity, With<MenuComponent>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
