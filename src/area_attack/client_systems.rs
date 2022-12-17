use bevy::prelude::*;
use bevy_egui::EguiContext;
use iyes_loopless::state::NextState;
use tap::Tap;

use crate::{
    common::Position,
    cursor::{Cursor, CursorBundle},
    load::{MineTextures, Textures},
    main_menu::standard_window,
    server::{ClientMessage, ClientSocket, MessageSocket, ServerMessage},
    singleplayer::minefield::{specific::CELL_SIZE, Minefield},
};

use super::{
    components::{ClientTile, ClientTileBundle},
    protocol::{AreaAttackRequest, AreaAttackUpdate},
    puppet::{PuppetCursor, PuppetCursorBundle, PuppetTable},
};

pub fn begin_game(mut ctx: ResMut<EguiContext>, sock: Option<ResMut<ClientSocket>>) {
    standard_window(&mut ctx, |ui| {
        ui.vertical_centered(|ui| {
            if ui.button("Begin game").clicked() {
                let _ = sock.unwrap().send_message(ClientMessage::Ingame {
                    data: rmp_serde::to_vec(&AreaAttackRequest::StartGame).unwrap(),
                });
            };
        })
    });
}

pub fn request_reveal(
    cursor: Query<(&Cursor, &Position)>,
    kb: Res<Input<KeyCode>>,
    mut sock: ResMut<ClientSocket>,
) {
    for (
        &Cursor {
            check_key,
            flag_key,
            ..
        },
        &position,
    ) in cursor.iter()
    {
        if kb.just_pressed(check_key) {
            // check.send(CheckCell(CursorPosition(position, owning_minefield)));
            sock.send_message(ClientMessage::Ingame {
                data: rmp_serde::to_vec(&AreaAttackRequest::Reveal(position)).unwrap(),
            });
        } else if kb.just_pressed(flag_key) {
            println!("Flagging not yet implemented")
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn listen_net(
    mut commands: Commands,
    mut sock: ResMut<ClientSocket>,
    mut tiles: Query<(Entity, &mut ClientTile)>,
    fields: Query<(Entity, &Minefield)>,
    cursors: Query<Entity, With<Cursor>>,
    tile_textures: Res<MineTextures>,
    misc_textures: Res<Textures>,
    mut puppet_map: ResMut<PuppetTable>,
    mut puppets: Query<(&mut PuppetCursor, &mut Position)>,
    mut field_id: Local<Option<Entity>>,
) {
    match sock.recv_message() {
        Some(Ok(AreaAttackUpdate::FieldShape(template))) => {
            // despawn previously constructed entities
            for e in tiles
                .iter()
                .map(|(e, _)| e)
                .chain(fields.iter().map(|(e, _)| e))
                .chain(cursors.iter())
            {
                commands.entity(e).despawn();
            }

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
                                    ..default()
                                };
                            }),
                        })
                        .id()
                },
                &template,
            );

            *field_id = Some(commands.spawn(field).id());
        }
        Some(Ok(AreaAttackUpdate::PlayerProperties {
            id,
            username, // TODO display username somehow
            color,
            position,
        })) => {
            puppet_map
                .entry(id)
                .and_modify(|&mut puppet| {
                    let (mut puppet, mut pos) = puppets.get_mut(puppet).unwrap();
                    puppet.0 = color.into();
                    *pos = position;
                })
                .or_insert_with(|| {
                    commands
                        .spawn(PuppetCursorBundle {
                            cursor: PuppetCursor(color.into()),
                            position,
                            sprite_bundle: SpriteBundle {
                                texture: misc_textures.cursor.clone(),
                                ..default()
                            },
                        })
                        .id()
                });
        }
        Some(Ok(AreaAttackUpdate::Reposition { id, position })) => {
            *(puppets.get_mut(puppet_map[&id]).unwrap().1) = position;
        }
        Some(Ok(AreaAttackUpdate::SelfChange { color })) => {
            commands.spawn(CursorBundle {
                cursor: Cursor::new(color.into(), field_id.unwrap()),
                position: Position::ZERO, // TODO: Randomize position
                texture: SpriteBundle {
                    texture: misc_textures.cursor.clone(),
                    transform: Transform {
                        translation: Position::ZERO.absolute(CELL_SIZE, CELL_SIZE).extend(3.0),
                        ..default()
                    },
                    ..default()
                },
            });
        }
        Some(Ok(AreaAttackUpdate::TileChanged { position, to })) => {
            *tiles.get_mut(fields.single().1[&position]).unwrap().1 = to;
        }
        Some(Ok(AreaAttackUpdate::Transition(to))) => commands.insert_resource(NextState(to)),
        _ => (),
    }
}

pub fn draw_mines(
    mut updated_tiles: Query<
        (&mut TextureAtlasSprite, &ClientTile),
        Or<(Added<ClientTile>, Changed<ClientTile>)>,
    >,
    own_cursor: Query<&Cursor>,
    puppet_map: ResMut<PuppetTable>,
    puppets: Query<&PuppetCursor>,
) {
    updated_tiles.for_each_mut(|(mut sprite, state)| {
        *sprite = match state {
            ClientTile::Unknown => {
                TextureAtlasSprite::new(9).tap_mut(|s| s.color = Color::default())
            }
            ClientTile::Owned {
                player,
                num_neighbors,
            } => TextureAtlasSprite::new(*num_neighbors as usize).tap_mut(|s| {
                s.color = if let Some(&PuppetCursor(color)) =
                    puppet_map.get(player).and_then(|e| puppets.get(*e).ok())
                {
                    color
                } else {
                    own_cursor.single().color
                }
            }),
            ClientTile::HardMine => {
                TextureAtlasSprite::new(0).tap_mut(|s| s.color = Color::default())
            }
        }
    })
}

pub fn send_position(
    pos: Query<&Position, (With<Cursor>, Changed<Position>)>,
    mut sock: ResMut<ClientSocket>,
) {
    for pos in pos.iter() {
        sock.send_message(ClientMessage::Ingame {
            data: rmp_serde::to_vec(&AreaAttackRequest::Position(*pos)).unwrap(),
        });
    }
}
