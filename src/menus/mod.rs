use bevy::prelude::*;
use iyes_loopless::{
    prelude::{AppLooplessStateExt, IntoConditionalSystem},
    state::NextState,
};

use crate::{load::Textures, SingleplayerState};

#[derive(Component)]
struct FailScreen;
#[derive(Component)]
struct SuccessScreen;

#[derive(Component)]
pub struct RetryButton;

fn fail_screen(mut commands: Commands, font_source: Res<Textures>) {
    create_screen(
        &mut commands,
        &font_source,
        "You failed with x% mines left".to_string(),
        RetryButton,
    );
}

fn retry(
    mut commands: Commands,
    ev: Query<&Interaction, (Changed<Interaction>, With<RetryButton>)>,
    ui: Query<Entity, With<FailScreen>>,
) {
    if ev.iter().any(|x| *x == Interaction::Clicked) {
        commands.insert_resource(NextState(SingleplayerState::PreGame));
        commands.entity(ui.single()).despawn_recursive();
    }
}

pub fn create_screen<T>(
    commands: &mut Commands,
    font_source: &Res<Textures>,
    message: String,
    marker: T,
) where
    T: Component,
{
    commands.spawn_bundle(UiCameraBundle::default());

    let text_style = || TextStyle {
        font_size: 40.0,
        color: Color::RED.into(),
        font: font_source.font.clone(),
    };

    // fail screen

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                margin: Rect::all(Val::Auto),
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .insert(FailScreen)
        .with_children(|parent| {
            // text
            parent
                .spawn_bundle(NodeBundle {
                    color: Color::WHITE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(message, text_style(), TextAlignment::default()),
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
                .insert(marker)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section("Button", text_style(), default()),
                        ..default()
                    });
                });
        });
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(SingleplayerState::GameFailed, fail_screen)
            .add_system(retry.run_in_state(SingleplayerState::GameFailed));
    }
}
