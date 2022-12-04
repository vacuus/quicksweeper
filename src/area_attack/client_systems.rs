use bevy::prelude::*;
use tap::Tap;

use crate::{
    common::Position,
    load::MineTextures,
    server::{ClientSocket, MessageSocket},
    singleplayer::minefield::specific::CELL_SIZE,
};

use super::{
    components::{ClientTile, ClientTileBundle},
    protocol::AreaAttackUpdate,
};

pub fn listen_events(
    mut commands: Commands,
    mut sock: ResMut<ClientSocket>,
    tiles: Query<Entity, With<ClientTile>>,
    textures: Res<MineTextures>,
) {
    match sock.recv_message() {
        Some(Ok(AreaAttackUpdate::FieldShape(template))) => {
            // despawn all existing tiles
            for tile in tiles.iter() {
                commands.entity(tile).despawn();
            }

            // spawn all received tiles
            for position @ Position { x, y } in template.decode() {
                commands.spawn(ClientTileBundle {
                    tile: ClientTile::Unknown,
                    position,
                    sprite: textures.empty().tap_mut(|b| {
                        b.transform = Transform {
                            translation: Vec3::new(
                                (x as f32) * CELL_SIZE,
                                (y as f32) * CELL_SIZE,
                                3.0,
                            ),
                            ..Default::default()
                        };
                    }),
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
