#![warn(clippy::all)]
#![allow(clippy::type_complexity)]

mod common;
mod cursor;
mod load;
mod main_menu;
mod minefield;
mod multiplayer;
mod singleplayer;
mod state;

use bevy::prelude::*;

use bevy_egui::EguiPlugin;
use main_menu::MainMenuPlugin;
pub use singleplayer::SingleplayerState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Quicksweeper".to_string(),
                ..default()
            },
            ..default()
        }))
        .add_plugin(EguiPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(singleplayer::SingleplayerMode)
        .add_plugin(multiplayer::MultiplayerMode)
        .add_plugin(common::QuicksweeperTypes)
        .add_plugin(load::LoadPlugin)
        .add_plugin(minefield::MinefieldPlugin)
        .run();
}
