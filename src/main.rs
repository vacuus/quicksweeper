#![warn(clippy::all)]
#![allow(clippy::type_complexity)]

mod common;
mod cursor;
mod minefield;
mod state;
mod load;
mod singleplayer;
mod multiplayer;
mod main_menu;

use bevy::prelude::*;

use bevy_egui::EguiPlugin;
use main_menu::MainMenuPlugin;
pub use singleplayer::SingleplayerState;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Quicksweeper".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(singleplayer::SingleplayerMode)
        .add_plugin(multiplayer::MultiplayerMode)
        .add_plugin(common::QuicksweeperTypes)
        .add_plugin(load::LoadPlugin)
        .add_plugin(minefield::MinefieldPlugin)
        .run();
}
