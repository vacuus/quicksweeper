#![cfg_attr(not(target = "server"), allow(dead_code, unused_imports))]

use std::{net::IpAddr, str::FromStr};

use bevy::prelude::*;

mod game;
mod protocol;
mod socket;

pub use game::*;
pub use protocol::*;
pub use socket::socket_pc::*;

pub struct ServerPlugin {
    pub address_name: Option<String>,
}

#[cfg(feature = "server")]
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        let address = self
            .address_name
            .clone()
            .map(|name| IpAddr::from_str(&name).unwrap())
            .unwrap_or_else(|| local_ip_address::local_ip().unwrap());

        app.insert_resource(OpenPort::generate(address))
            .add_event::<IngameEvent>()
            .add_event::<ConnectionSwitch>()
            .add_system_to_stage(CoreStage::PostUpdate, delay_hierarchy_events)
            .add_system(receive_connections)
            .add_system(upgrade_connections)
            .add_system(clean_dead_connections)
            .add_system(try_send_repeated)
            .add_system(clean_empty_games)
            .add_system(game_messages)
            .add_system(server_messages);
    }
}

#[cfg(feature = "server")]
pub fn server_app(address_name: Option<String>) -> App {
    use std::time::Duration;

    use bevy::app::{RunMode, ScheduleRunnerSettings};

    use crate::{area_attack, common, load, minefield, registry::GameRegistry, server};

    let mut app = App::new();
    app.init_resource::<GameRegistry>()
        // run the server at 60 hz
        .insert_resource(ScheduleRunnerSettings {
            run_mode: RunMode::Loop {
                wait: Some(Duration::from_secs(1).div_f32(60.)),
            },
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
