use bevy::prelude::*;
use bevy_egui::EguiContext;
use iyes_loopless::state::{CurrentState, NextState};
use tap::Tap;

use crate::{
    common::Position,
    cursor::{Bindings, Cursor, CursorBundle},
    load::Textures,
    main_menu::standard_window,
    minefield::{query::MinefieldQuery, specific::TILE_SIZE, Minefield},
    server::{ClientMessage, Connection},
};

use super::{
    components::{ClientTile, ClientTileBundle, FreezeTimer, FreezeTimerDisplay},
    protocol::{AreaAttackRequest, AreaAttackUpdate},
    puppet::{PuppetCursor, PuppetCursorBundle, PuppetTable},
    states::AreaAttack,
};

pub fn begin_game(mut ctx: ResMut<EguiContext>, sock: Option<ResMut<Connection>>) {
    standard_window(&mut ctx, |ui| {
        ui.vertical_centered(|ui| {
            if ui.button("Begin game").clicked() {
                sock.unwrap().repeat_send_unchecked(ClientMessage::Ingame {
                    data: rmp_serde::to_vec(&AreaAttackRequest::StartGame).unwrap(),
                });
            };
        })
    });
}

pub fn request_reveal(
    cursor: Query<&Position, With<Cursor>>,
    keybinds: Res<Bindings>,
    kb: Res<Input<KeyCode>>,
    mut sock: ResMut<Connection>,
    state: Res<CurrentState<AreaAttack>>,
    mut field: MinefieldQuery<&mut ClientTile>,
    puppet_table: Res<PuppetTable>,
) {
    for &position in cursor.iter() {
        let mut field = field.get_single().unwrap();
        if kb.just_pressed(keybinds.check) {
            match field.get(position).unwrap() {
                ClientTile::Unknown => {
                    sock.send_logged(ClientMessage::Ingame {
                        data: rmp_serde::to_vec(&AreaAttackRequest::Reveal(position)).unwrap(),
                    });
                }
                ClientTile::Owned {
                    player,
                    num_neighbors,
                } => {
                    if !puppet_table.contains_key(player) {
                        // counts both flags and known mines
                        let marked_count = field
                            .iter_neighbors(position)
                            .filter(|tile| matches!(tile, ClientTile::Flag | ClientTile::HardMine))
                            .count() as u8;

                        if marked_count == *num_neighbors {
                            for (position, tile) in field.iter_neighbors_enumerated(position) {
                                if !matches!(tile, ClientTile::Flag) {
                                    sock.send_logged(ClientMessage::Ingame {
                                        data: rmp_serde::to_vec(&AreaAttackRequest::Reveal(
                                            position,
                                        ))
                                        .unwrap(),
                                    });
                                }
                            }
                        }
                    }
                }
                _ => (), // do nothing, since these tiles semantically contains mines
            }
        } else if kb.just_pressed(keybinds.flag)
            && !matches!(
                state.0,
                // truthfully the last of these three should be impossible, but check it anyway
                AreaAttack::Selecting | AreaAttack::Finishing | AreaAttack::Inactive
            )
        {
            if let Some(mut tile) = field.get_mut(position) {
                match *tile {
                    ClientTile::Unknown => *tile = ClientTile::Flag,
                    ClientTile::Flag => *tile = ClientTile::Unknown,
                    _ => (), // do nothing, since these tiles are nonsensical to flag
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)] // TODO Split this up
pub fn listen_net(
    mut commands: Commands,
    mut sock: ResMut<Connection>,
    mut tiles: Query<(Entity, &mut ClientTile)>,
    fields: Query<(Entity, &Minefield)>,
    cursors: Query<Entity, With<Cursor>>,
    textures: Res<Textures>,
    mut puppet_map: ResMut<PuppetTable>,
    mut puppets: Query<(&mut PuppetCursor, &mut Position)>,
    mut field_id: Local<Option<Entity>>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    mut freeze_timer: ResMut<FreezeTimer>,
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
                            sprite: textures.empty_mine().tap_mut(|b| {
                                b.transform = Transform {
                                    translation: position
                                        .absolute(TILE_SIZE, TILE_SIZE)
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
            username,
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
                    let name = commands
                        .spawn(Text2dBundle {
                            text: Text::from_section(
                                username,
                                TextStyle {
                                    font: textures.roboto.clone(),
                                    font_size: 10.0,
                                    color: color.into(),
                                },
                            ),
                            transform: Transform {
                                translation: Vec3 {
                                    x: 10.0,
                                    y: 10.0,
                                    z: 0.0,
                                },
                                ..default()
                            },
                            ..default()
                        })
                        .id();

                    commands
                        .spawn(PuppetCursorBundle {
                            cursor: PuppetCursor(color.into()),
                            position,
                            sprite_bundle: SpriteBundle {
                                texture: textures.cursor.clone(),
                                ..default()
                            },
                        })
                        .add_child(name)
                        .id()
                });
        }
        Some(Ok(AreaAttackUpdate::Reposition { id, position })) => {
            *(puppets.get_mut(puppet_map[&id]).unwrap().1) = position;
        }
        Some(Ok(AreaAttackUpdate::SelfChange { color, position })) => {
            let translation = position.absolute(TILE_SIZE, TILE_SIZE).extend(3.0);
            camera.single_mut().translation = translation;
            commands.spawn(CursorBundle {
                cursor: Cursor::new(color.into(), field_id.unwrap()),
                position,
                texture: SpriteBundle {
                    texture: textures.cursor.clone(),
                    transform: Transform {
                        translation,
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
        Some(Ok(AreaAttackUpdate::Freeze)) => freeze_timer.reset(),
        _ => (),
    }
}

pub fn create_freeze_timer(mut commands: Commands, textures: Res<Textures>) {
    commands
        .spawn(
            TextBundle::from_section(
                "0.00",
                TextStyle {
                    font: textures.roboto.clone(),
                    font_size: 32.0,
                    color: Color::RED,
                },
            )
            .tap_mut(|t| t.visibility.is_visible = false),
        )
        .insert(FreezeTimerDisplay);
}

pub fn freeze_timer(
    mut timer: ResMut<FreezeTimer>,
    time: Res<Time>,
    mut timer_text: Query<(&mut Text, &mut Visibility), With<FreezeTimerDisplay>>,
) {
    let seconds = timer.tick(time.delta()).remaining_secs();
    let (mut text, mut visibility) = timer_text.single_mut();

    if timer.finished() {
        visibility.is_visible = false;
    } else {
        visibility.is_visible = true;
        text.sections[0].value = seconds.to_string()
    }
}

pub fn draw_tiles(
    mut updated_tiles: Query<
        (&mut TextureAtlasSprite, &ClientTile),
        Or<(Added<ClientTile>, Changed<ClientTile>)>,
    >,
    own_cursor: Query<&Cursor>,
    puppet_map: ResMut<PuppetTable>,
    puppets: Query<&PuppetCursor>,
) {
    let own_color = own_cursor.get_single().map(|c| c.color).map_err(|_| ());
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
                    own_color.unwrap()
                }
            }),
            ClientTile::HardMine => {
                TextureAtlasSprite::new(11).tap_mut(|s| s.color = Color::default())
            }
            ClientTile::Flag => {
                TextureAtlasSprite::new(10).tap_mut(|s| s.color = own_color.unwrap())
            }
        }
    })
}

pub fn send_position(
    pos: Query<&Position, (With<Cursor>, Or<(Added<Position>, Changed<Position>)>)>,
    mut sock: ResMut<Connection>,
) {
    for pos in pos.iter() {
        sock.send_logged(ClientMessage::Ingame {
            data: rmp_serde::to_vec(&AreaAttackRequest::Position(*pos)).unwrap(),
        });
    }
}
