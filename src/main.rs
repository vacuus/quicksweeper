#![warn(clippy::all)]
#![allow(clippy::type_complexity)] // this lint marks type signatures of queries as too long, which is unnecessary

mod area_attack;
mod common;
mod cser;
mod cursor;
mod load;
mod main_menu;
mod minefield;
mod registry;
mod server;
mod server_v2;
mod singleplayer;
mod state;

use bevy::prelude::*;

use bevy_egui::EguiPlugin;
use clap::{Parser, Subcommand};
use cursor::CursorPlugin;
use main_menu::MainMenuPlugin;
use registry::GameRegistry;
pub use singleplayer::Singleplayer;

#[derive(Parser)]
#[command(author, version)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    mode: Option<Mode>,
}

#[derive(Subcommand)]
enum Mode {
    Client,
    Server { address: Option<String> },
}

fn client_app() -> App {
    let mut app = App::new();
    app.init_resource::<GameRegistry>()
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
        .add_plugin(minefield::MinefieldPlugin)
        // gamemodes
        .add_plugin(singleplayer::SingleplayerMode)
        .add_plugin(area_attack::AreaAttackClient)
        // framerate
        .add_plugin(bevy_framepace::FramepacePlugin)
        .add_startup_system(framerate_limit);
    app
}

fn framerate_limit(mut settings: ResMut<bevy_framepace::FramepaceSettings>) {
    settings.limiter = bevy_framepace::Limiter::from_framerate(30.0);
}

fn main() {
    match Args::parse().mode {
        None | Some(Mode::Client) => client_app(),
        #[allow(unused)]
        Some(Mode::Server { address }) => {
            #[cfg(feature = "server")]
            {
                server::server_app(address)
            }
            #[cfg(not(feature = "server"))]
            {
                panic!("Server feature was not enabled for this build, so server initialization is impossible.")
            }
        }
    }
    .run();
}
