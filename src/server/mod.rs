use std::{net::IpAddr, str::FromStr};

use bevy::prelude::*;

use self::sockets::*;

mod game;
mod protocol;
mod sockets;

pub use game::*;
pub use protocol::*;
pub use sockets::{ClientSocket, Connection, ConnectionInfo, MessageSocket};

pub struct ServerPlugin {
    pub address_name: Option<String>,
}

fn test_added(q: Query<&GameMarker, Added<GameMarker>>) {
    for item in q.iter() {
        println!("added: {item:?}")
    }
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        let address = self
            .address_name
            .clone()
            .map(|name| IpAddr::from_str(&name).unwrap())
            .unwrap_or_else(|| local_ip_address::local_ip().unwrap());

        app.insert_resource(OpenPort::generate(&address))
            .add_event::<IngameEvent>()
            .add_event::<ConnectionSwitch>()
            .add_system_to_stage(CoreStage::PostUpdate, delay_hierarchy_events)
            .add_system(test_added)
            .add_system(receive_connections)
            .add_system(upgrade_connections)
            .add_system(game_messages)
            .add_system(server_messages);
    }
}
