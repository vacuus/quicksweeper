#![allow(dead_code)]

mod minefield;
mod textures;
mod cursor;
mod common;

use bevy::prelude::*;
use common::CheckCell;
use iyes_loopless::prelude::AppLooplessStateExt;
use rand::prelude::*;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
enum AppState {
    Menu,
    Game,
    GameFailed,
}

fn main() {
    App::new()
        .add_loopless_state(AppState::Game)
        .insert_resource(StdRng::from_entropy())
        .add_event::<CheckCell>()
        .add_plugins(DefaultPlugins)
        .add_startup_system(textures::load_textures)
        .add_plugin(cursor::CursorPlugin)
        .add_plugin(minefield::MinefieldPlugin)
        .run();
}
