#![allow(dead_code)]

mod minefield;
mod textures;
mod cursor;
mod common;
mod state;

use bevy::prelude::*;
use common::CheckCell;
use iyes_loopless::prelude::AppLooplessStateExt;
use rand::prelude::*;

pub use state::AppState;

fn main() {
    App::new()
        .add_loopless_state(AppState::PreGame)
        .insert_resource(StdRng::from_entropy())
        .add_event::<CheckCell>()
        .add_plugins(DefaultPlugins)
        .add_startup_system(textures::load_textures)
        .add_plugin(cursor::CursorPlugin)
        .add_plugin(minefield::MinefieldPlugin)
        .run();
}
