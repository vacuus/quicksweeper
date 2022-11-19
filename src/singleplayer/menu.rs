use bevy::prelude::*;
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionSet},
    state::{CurrentState, NextState},
};

use crate::{load::Textures, main_menu::MenuState, minefield::Minefield, SingleplayerState};

#[derive(Component)]
struct CompleteScreen;

#[derive(Component)]
struct RetryButton;
#[derive(Component)]
struct MainMenuButton;

fn fail_screen(mut commands: Commands, font_source: Res<Textures>, minefield: Query<&Minefield>) {
    let remaining = minefield.single().remaining_blank();

    create_screen(
        &mut commands,
        &font_source,
        format!("You failed with {remaining} tiles left"),
    );
}

fn success_screen(mut commands: Commands, font_source: Res<Textures>) {
    create_screen(
        &mut commands,
        &font_source,
        "Congratulations!".to_string(), // TODO: Calculate score
    );
}

fn retry(
    mut commands: Commands,
    ev: Query<&Interaction, (Changed<Interaction>, With<RetryButton>)>,
    // ui: Query<Entity, With<CompleteScreen>>,
) {
    if ev.iter().any(|x| *x == Interaction::Clicked) {
        commands.insert_resource(NextState(SingleplayerState::PreGame));
        // commands.entity(ui.single()).despawn_recursive();
    }
}

fn main_menu(
    mut commands: Commands,
    ev: Query<&Interaction, (Changed<Interaction>, With<MainMenuButton>)>,
) {
    if ev.iter().any(|x| *x == Interaction::Clicked) {
        commands.insert_resource(NextState(SingleplayerState::Inactive));
        commands.insert_resource(NextState(MenuState::MainMenu));
    }
}

fn close_menu(mut commands: Commands, ui: Query<Entity, With<CompleteScreen>>) {
    ui.for_each(|x| commands.entity(x).despawn_recursive())
}

pub fn create_screen(commands: &mut Commands, font_source: &Res<Textures>, message: String) {
    let text_style = || TextStyle {
        font_size: 40.0,
        color: Color::RED,
        font: font_source.font.clone(),
    };

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(CompleteScreen)
        .with_children(|parent| {
            // text
            parent
                .spawn_bundle(NodeBundle {
                    color: Color::WHITE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::from_section(message, text_style()),
                        ..default()
                    });
                });
            // button
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    color: Color::MIDNIGHT_BLUE.into(),
                    ..default()
                })
                .insert(RetryButton)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::from_section("Retry", text_style()),
                        ..default()
                    });
                });
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    color: Color::MIDNIGHT_BLUE.into(),
                    ..default()
                })
                .insert(MainMenuButton)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::from_section("Main Menu", text_style()),
                        ..default()
                    });
                });
        });
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(SingleplayerState::GameFailed, fail_screen)
            .add_enter_system(SingleplayerState::GameSuccess, success_screen)
            .add_system_set(
                ConditionSet::new()
                    .run_if(|state: Res<CurrentState<SingleplayerState>>| {
                        [
                            SingleplayerState::GameFailed,
                            SingleplayerState::GameSuccess,
                        ]
                        .contains(&state.0)
                    })
                    .with_system(retry)
                    .with_system(main_menu)
                    .into(),
            )
            .add_exit_system(SingleplayerState::GameFailed, close_menu)
            .add_exit_system(SingleplayerState::GameSuccess, close_menu);
    }
}
