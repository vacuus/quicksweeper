use bevy::prelude::*;

use self::{
    protocol::{ClientMessage, ServerMessage},
    sockets::*,
};

mod game;
mod protocol;
mod sockets;

pub use game::*;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OpenPort::generate())
            .init_resource::<Events<ServerMessage>>() // communicate_clients is the sole consumer of ServerMessage
            .add_event::<ClientMessage>()
            .add_system(receive_connections)
            .add_system(upgrade_connections)
            .add_system(communicate_clients);
    }
}
