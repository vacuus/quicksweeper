use bevy::prelude::*;

use crate::area_attack::AREA_ATTACK_UUID;

use self::sockets::*;

mod game;
mod protocol;
mod sockets;

pub use game::*;
pub use protocol::*;
pub use sockets::{ClientSocket, MessageSocket};

pub struct ServerPlugin;

fn test_added(q: Query<&GameMarker, Added<GameMarker>>) {
    for item in q.iter() {
        println!("added: {item:?}")
    }
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        println!(
            "{:X?}",
            rmp_serde::to_vec(&ClientMessage::Create {
                game: GameMarker(AREA_ATTACK_UUID),
                data: Vec::new()
            })
        );

        app.insert_resource(OpenPort::generate())
            .add_event::<IngameEvent>()
            .add_system(test_added)
            .add_system(receive_connections)
            .add_system(upgrade_connections)
            .add_system(game_messages)
            .add_system(server_messages);
    }
}
