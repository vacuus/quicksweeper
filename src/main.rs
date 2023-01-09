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
mod singleplayer;
mod state;

use std::time::Duration;

use bevy::{
    app::{RunMode, ScheduleRunnerSettings},
    diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*, winit::WinitSettings,
};

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

fn server_app(address_name: Option<String>) -> App {
    let mut app = App::new();
    app.init_resource::<GameRegistry>()
        // run the server at 60 hz
        .insert_resource(ScheduleRunnerSettings {
            run_mode: RunMode::Loop { wait: Some(Duration::from_secs(1).div_f32(60.))},
        })
        .add_plugins(MinimalPlugins)
        .add_plugin(AssetPlugin::default())
        .add_plugin(HierarchyPlugin)
        .add_plugin(common::QuicksweeperTypes)
        .add_plugin(server::ServerPlugin { address_name })
        .add_plugin(load::ServerLoad)
        .add_plugin(minefield::MinefieldPlugin)
        // gamemodes
        .add_plugin(area_attack::AreaAttackServer);
    app
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
        Some(Mode::Server { address }) => server_app(address),
    }
    .run();
}
