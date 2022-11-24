use bevy::prelude::*;

use self::sockets::*;

mod game;
mod sockets;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OpenPort::generate())
            .add_system(receive_client)
            .add_system(listen_clients)
            ;
    }
}
