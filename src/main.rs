mod common;
mod cursor;
mod minefield;
mod state;
mod load;

use bevy::prelude::*;

pub use state::SingleplayerState;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Quicksweeper".to_string(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(common::QuicksweeperTypes)
        .add_plugin(load::LoadPlugin)
        .add_plugin(cursor::CursorPlugin)
        .add_plugin(minefield::MinefieldPlugin)
        .run();
}
