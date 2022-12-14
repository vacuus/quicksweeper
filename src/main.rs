#![warn(clippy::all)]
#![allow(clippy::type_complexity)] // this lint marks type signatures of queries as too long, which is unnecessary

mod area_attack;
mod common;
mod cser;
mod cursor;
mod load;
mod main_menu;
mod registry;
mod server;
mod singleplayer;
mod state;

use bevy::prelude::*;

use bevy_egui::EguiPlugin;
use cursor::CursorPlugin;
use main_menu::MainMenuPlugin;
use registry::GameRegistry;
pub use singleplayer::SingleplayerState;

fn run_server() {
    App::new()
        .init_resource::<GameRegistry>()
        // .add_plugins(DefaultPlugins)
        .add_plugins(MinimalPlugins)
        // .add_plugin(CorePlugin::default())
        // .add_plugin(TimePlugin::default())
        .add_plugin(AssetPlugin::default())
        .add_plugin(HierarchyPlugin)
        .add_plugin(common::QuicksweeperTypes)
        .add_plugin(server::ServerPlugin)
        .add_plugin(load::ServerLoad)
        .add_plugin(singleplayer::minefield::MinefieldPlugin)
        // gamemodes
        .add_plugin(area_attack::AreaAttackServer)
        .run();
}

fn run_client() {
    App::new()
        .init_resource::<GameRegistry>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Quicksweeper".to_string(),
                ..default()
            },
            ..default()
        }))
        .add_plugin(EguiPlugin)
        .add_plugin(CursorPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(common::QuicksweeperTypes)
        .add_plugin(load::ClientLoad)
        .add_plugin(singleplayer::minefield::MinefieldPlugin)
        // gamemodes
        .add_plugin(singleplayer::SingleplayerMode)
        .add_plugin(area_attack::AreaAttackClient)
        .run();
}

fn main() {
    if matches!(std::env::args().nth(1), Some(x) if x == "srv") {
        run_server()
    } else {
        run_client()
    }
}
