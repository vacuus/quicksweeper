#![allow(dead_code)]

mod common;
mod cursor;
mod minefield;
mod state;
mod textures;

use bevy::prelude::*;
use common::CheckCell;
use iyes_loopless::prelude::AppLooplessStateExt;
use rand::prelude::*;

pub use state::AppState;

fn main() {
    App::new()
        .add_loopless_state(AppState::Loading)
        .insert_resource(StdRng::from_entropy())
        .add_event::<CheckCell>()
        .add_plugins(DefaultPlugins)
        .add_startup_system(textures::load_textures)
        .add_plugin(cursor::CursorPlugin)
        .add_plugin(minefield::MinefieldPlugin)
        .run();
}
