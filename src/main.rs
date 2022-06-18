#![warn(clippy::all)]
#![allow(clippy::type_complexity)]

mod common;
mod cursor;
mod minefield;
mod state;
mod load;
mod singleplayer;

use bevy::prelude::*;

pub use singleplayer::SingleplayerState;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Quicksweeper".to_string(),
            ..Default::default()
        })
        .add_plugin(singleplayer::SingleplayerMode)
        .add_plugins(DefaultPlugins)
        .add_plugin(common::QuicksweeperTypes)
        .add_plugin(load::LoadPlugin)
        .add_plugin(minefield::MinefieldPlugin)
        .run();
}
