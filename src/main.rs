#![warn(clippy::all)]
#![allow(clippy::type_complexity)]

mod common;
mod cursor;
mod minefield;
mod state;
mod load;
mod menus;
mod singleplayer;

use bevy::prelude::*;

pub use singleplayer::SingleplayerState;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Quicksweeper".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(common::QuicksweeperTypes)
        .add_plugin(menus::MenuPlugin)
        .add_plugin(load::LoadPlugin)
        .add_plugin(cursor::CursorPlugin)
        .add_plugin(minefield::MinefieldPlugin)
        .add_plugin(singleplayer::SingleplayerMode)
        .run();
}
