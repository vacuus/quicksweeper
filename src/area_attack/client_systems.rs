use bevy::{ecs::system::SystemChangeTick, prelude::*};
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
                            .filter(|tile| matches!(tile, ClientTile::Flag | ClientTile::Mine))
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

pub fn listen_net(mut events: EventWriter<AreaAttackUpdate>, mut sock: ResMut<Connection>) {
    while let Some(Ok(m)) = sock.recv_message() {
        events.send(m)
    }
}

/// Despite its name, this system is also used to create a new field if it didn't exist before
pub fn reset_field(
    mut events: EventReader<AreaAttackUpdate>,
    mut commands: Commands,
    textures: Res<Textures>,
    old_entities: Query<Entity, Or<(With<ClientTile>, With<Minefield>, With<Cursor>)>>,
) {
    for ev in events.iter() {
        if let AreaAttackUpdate::FieldShape(template) = ev {
            // despawn previously constructed entities
            for e in old_entities.iter() {
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
                                b.transform.translation =
                                    position.absolute(TILE_SIZE, TILE_SIZE).extend(3.0);
                            }),
                        })
                        .id()
                },
                template,
            );

            commands.spawn(field);
        }
    }
}

pub fn player_update(
    mut events: EventReader<AreaAttackUpdate>,
    mut commands: Commands,
    mut puppet_map: ResMut<PuppetTable>,
    mut puppets: Query<(&mut PuppetCursor, &mut Position)>,
    textures: Res<Textures>,
) {
    for ev in events.iter() {
        if let AreaAttackUpdate::PlayerProperties {
            id,
            username,
            color,
            position,
        } = ev
        {
            puppet_map
                .entry(*id)
                .and_modify(|&mut puppet| {
                    let (mut puppet, mut pos) = puppets.get_mut(puppet).unwrap();
                    puppet.0 = (*color).into();
                    *pos = *position;
                })
                .or_insert_with(|| {
                    let name = commands
                        .spawn(Text2dBundle {
                            text: Text::from_section(
                                username,
                                TextStyle {
                                    font: textures.roboto.clone(),
                                    font_size: 10.0,
                                    color: (*color).into(),
                                },
                            ),
                            transform: Transform::from_xyz(10.0, 10.0, 0.0),
                            ..default()
                        })
                        .id();

                    commands
                        .spawn(PuppetCursorBundle {
                            cursor: PuppetCursor((*color).into()),
                            position: *position,
                            sprite_bundle: SpriteBundle {
                                texture: textures.cursor.clone(),
                                ..default()
                            },
                        })
                        .add_child(name)
                        .id()
                });
        }
    }
}

pub fn self_update(
    mut events: EventReader<AreaAttackUpdate>,
    mut commands: Commands,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    textures: Res<Textures>,
    field: Query<Entity, With<Minefield>>,
    mut save_event: Local<Option<AreaAttackUpdate>>,
) {
    if let Some(AreaAttackUpdate::SelfChange { color, position }) =
        std::mem::replace(&mut *save_event, None).or_else(|| {
            events
                .iter()
                .filter(|x| matches!(x, AreaAttackUpdate::SelfChange { .. }))
                .last()
                .cloned()
        })
    {
        if let Ok(field) = field.get_single() {
            let translation = position.absolute(TILE_SIZE, TILE_SIZE).extend(3.0);
            camera.single_mut().translation = translation;
            commands.spawn(CursorBundle {
                cursor: Cursor::new((color).into(), field),
                position,
                texture: SpriteBundle {
                    texture: textures.cursor.clone(),
                    transform: Transform::from_translation(translation),
                    ..default()
                },
            });
        } else {
            *save_event = Some(AreaAttackUpdate::SelfChange { color, position })
        }
    }
}

pub fn puppet_control(
    mut events: EventReader<AreaAttackUpdate>,
    mut puppets: Query<(&mut PuppetCursor, &mut Position)>,
    mut field: MinefieldQuery<&mut ClientTile>,
    puppet_map: ResMut<PuppetTable>,
    current_tick: SystemChangeTick,
) {
    let mut puppeted = 0;
    for ev in events.iter() {
        match ev {
            AreaAttackUpdate::Reposition { id, position } => {
                *(puppets.get_mut(puppet_map[id]).unwrap().1) = *position;
            }
            AreaAttackUpdate::TileChanged { position, to } => {
                puppeted += 1;
                *field.get_single().unwrap().get_mut(*position).unwrap() = *to
            }
            _ => (),
        }
    }

    if puppeted > 0 {
        println!("Received puppet command on {}", current_tick.change_tick())
    }
}

pub fn state_transitions(
    mut events: EventReader<AreaAttackUpdate>,
    mut freeze_timer: ResMut<FreezeTimer>,
    mut commands: Commands,
) {
    for ev in events.iter() {
        match ev {
            AreaAttackUpdate::Transition(to) => commands.insert_resource(NextState(*to)),
            AreaAttackUpdate::Freeze => freeze_timer.reset(),
            _ => (),
        }
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
            ClientTile::Mine => TextureAtlasSprite::new(11).tap_mut(|s| s.color = Color::default()),
            ClientTile::Flag => {
                TextureAtlasSprite::new(10).tap_mut(|s| s.color = own_color.unwrap())
            }
            ClientTile::Destroyed => TextureAtlasSprite::new(9).tap_mut(|s| s.color = Color::BLACK),
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
