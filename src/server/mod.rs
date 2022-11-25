use bevy::prelude::*;

use self::sockets::*;

mod game;
mod sockets;

pub use game::{QuicksweeperGame, GameDescriptor};

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OpenPort::generate())
            .add_system(receive_connections)
            .add_system(upgrade_connections)
            .add_system(listen_clients)
            ;
    }
}
