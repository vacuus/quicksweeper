use bevy::prelude::*;

use crate::server::{ClientSocket, MessageSocket};

use super::{
    components::{ClientTile, ClientTileBundle},
    protocol::AreaAttackUpdate,
};

fn start_game(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn listen_events(
    mut commands: Commands,
    mut sock: ResMut<ClientSocket>,
    tiles: Query<Entity, With<ClientTile>>,
) {
    match sock.recv_message() {
        Some(Ok(AreaAttackUpdate::FieldShape(template))) => {
            // despawn all existing tiles
            for tile in tiles.iter() {
                commands.entity(tile).despawn();
            }

            // spawn all tiles sent
            for position in template.decode() {
                commands.spawn(ClientTileBundle {
                    tile: ClientTile::Unknown,
                    position,
                });
            }
        }
        Some(Ok(AreaAttackUpdate::PlayerModified {
            id,
            username,
            color,
        })) => {}
        Some(Ok(AreaAttackUpdate::SelfModified { color })) => {}
        Some(Ok(AreaAttackUpdate::TileChanged { position, to })) => {}
        _ => (),
    }
}

// fn draw_minefield(
//     cells: Query<
// )
