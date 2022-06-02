#![allow(dead_code)]

mod common;
mod cursor;
mod minefield;
mod state;
mod textures;

use bevy::prelude::*;

pub use state::AppState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(common::QuicksweeperTypes)
        .add_startup_system(textures::load_textures)
        .add_plugin(cursor::CursorPlugin)
        .add_plugin(minefield::MinefieldPlugin)
        .run();
}
