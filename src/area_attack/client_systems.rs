use bevy::prelude::*;
use tap::Tap;

use crate::{
    common::Position,
    cursor::{Cursor, CursorPosition},
    load::{MineTextures, Textures},
    server::{ClientSocket, MessageSocket},
    singleplayer::minefield::{specific::CELL_SIZE, Minefield},
};

use super::{
    components::{ClientTile, ClientTileBundle},
    protocol::AreaAttackUpdate,
};

pub fn listen_events(
    mut commands: Commands,
    mut sock: ResMut<ClientSocket>,
    tiles: Query<Entity, With<ClientTile>>,
    fields: Query<Entity, With<Minefield>>,
    cursors: Query<Entity, With<Cursor>>,
    tile_textures: Res<MineTextures>,
    misc_textures: Res<Textures>,
) {
    match sock.recv_message() {
        Some(Ok(AreaAttackUpdate::FieldShape(template))) => {
            // despawn previously constructed entities
            for e in tiles.iter().chain(fields.iter()).chain(cursors.iter()) {
                commands.entity(e).despawn();
            }

            let init_position = template.center().unwrap_or(Position::ZERO);

            // spawn all received tiles
            let field = Minefield::new_shaped(
                |&position| {
                    commands
                        .spawn(ClientTileBundle {
                            tile: ClientTile::Unknown,
                            position,
                            sprite: tile_textures.empty().tap_mut(|b| {
                                b.transform = Transform {
                                    translation: position
                                        .absolute(CELL_SIZE, CELL_SIZE)
                                        .extend(3.0),
                                    ..Default::default()
                                };
                            }),
                        })
                        .id()
                },
                &template,
            );

            let field_id = commands.spawn(field).id();

            commands
                .spawn(SpriteBundle {
                    texture: misc_textures.cursor.clone(),
                    transform: Transform {
                        translation: init_position.absolute(32.0, 32.0).extend(3.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Cursor::new(CursorPosition(init_position, field_id)));
        }
        Some(Ok(AreaAttackUpdate::PlayerProperties{
            id,
            username,
            color,
            position,
        })) => {}
        Some(Ok(AreaAttackUpdate::SelfChange { color })) => {}
        Some(Ok(AreaAttackUpdate::TileChanged { position, to })) => {}
        _ => (),
    }
}
