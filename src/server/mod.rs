use bevy::prelude::*;

use self::{sockets::*, protocol::{ServerMessage, ClientData}};

mod game;
mod protocol;
mod sockets;

pub use game::*;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OpenPort::generate())
            .add_event::<ServerMessage>()
            .add_event::<ClientData>()
            .add_system(receive_connections)
            .add_system(upgrade_connections)
            .add_system(listen_clients);
    }
}
