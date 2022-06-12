use bevy::prelude::*;
use iyes_loopless::prelude::AppLooplessStateExt;

use crate::{load::Textures, SingleplayerState};

#[derive(Component)]
struct FailScreen;
#[derive(Component)]
struct SuccessScreen;

fn init(mut commands: Commands, font_source: Res<Textures>) {
    commands.spawn_bundle(UiCameraBundle::default());

    // fail screen

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                margin: Rect::all(Val::Auto),
                ..default()
            },
            visibility: Visibility { is_visible: false },
            ..default()
        })
        .insert(FailScreen)
        .with_children(|parent| {
            // text
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Failed with x% tiles left",
                    TextStyle {
                        font_size: 40.0,
                        color: Color::rgb(0.1, 0.1, 0.1),
                        font: font_source.font.clone(),
                    },
                    TextAlignment::default(),
                ),
                ..default()
            });
            // button
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        ..default()
                    },
                    color: Color::rgb(0.1, 0.1, 0.1).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Button",
                            TextStyle {
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                                font: font_source.font.clone(),
                            },
                            default(),
                        ),
                        ..default()
                    });
                });
        });
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(SingleplayerState::PreGame, init);
    }
}
