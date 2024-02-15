mod game;
mod state;

use bevy::prelude::*;
use iyes_loopless::prelude::*;
use std::time::Duration;

use state::AppState;


fn main() {
    let mut fixed_update = SystemStage::parallel();
    fixed_update.add_system(
        game::handle_input
            .run_in_state(AppState::InGame)
            .label("handle_input"),
    );

    fixed_update.add_system(
        game::update_velocity
            .run_in_state(AppState::InGame)
            .label("update_velocity"),
    );
    fixed_update.add_system(
        game::apply_velocity
            .run_in_state(AppState::InGame)
            .label("apply_velocity")
            .after("update_velocity"),
    );

    fixed_update.add_system(
        game::use_brain
            .run_in_state(AppState::InGame)
            .label("use_brain")
            .after("apply_velocity"),
    );
    fixed_update.add_system(
        game::eat_berries
            .run_in_state(AppState::InGame)
            .label("eat_berries")
            .after("use_brain"),
    );
    fixed_update.add_system(
        game::spawn_berries
            .run_in_state(AppState::InGame)
            .label("spawn_berries")
            .after("eat_berries"),
    );

    App::new()
        .add_plugins(DefaultPlugins)
        .add_loopless_state(AppState::Menu)
        .add_stage_before(
            CoreStage::Update,
            "FixedUpdate",
            FixedTimestepStage::from_stage(Duration::from_millis(game::TIME_STEP_MS), fixed_update),
        )
        .add_enter_system(AppState::Menu, setup_menu)
        .add_system_set(
            ConditionSet::new()
                .run_in_state(AppState::Menu)
                .with_system(menu)
                .into(),
        )
        .add_exit_system(AppState::Menu, cleanup_menu)
        .add_enter_system(AppState::InGame, game::setup_game)
        .add_exit_system(AppState::InGame, game::cleanup_game)
        .run();
}

struct MenuData {
    button_entity: Entity,
}


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

