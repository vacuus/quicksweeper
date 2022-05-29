#![allow(dead_code)]

mod minefield;
mod textures;

use bevy::prelude::*;
use iyes_loopless::prelude::AppLooplessStateExt;
use rand::prelude::*;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
enum AppState {
    Menu,
    Game,
}

fn main() {
    App::new()
        .add_loopless_state(AppState::Game)
        .insert_resource(StdRng::from_entropy())
        .add_plugins(DefaultPlugins)
        .add_startup_system(textures::load_textures)
        .add_plugin(minefield::MinefieldPlugin)
        .run();
}
