use bevy::{gltf::Gltf, prelude::*, scene::SceneInstance};
use bevy_egui::EguiContext;
use iyes_loopless::state::{CurrentState, NextState};
use tap::Tap;

use crate::{
    common::{NeedsMaterial, Position, Vec2Ext},
    cursor::{Bindings, Cursor, CursorBundle, MainCursorMaterial},
    load::Textures,
    main_menu::standard_window,
    minefield::{query::MinefieldQuery, specific::TILE_SIZE, Minefield},
    server::{ClientMessage, CommonConnection as Connection},
};

use super::{
    components::{ClientTile, ClientTileBundle, FreezeTimer, FreezeTimerDisplay},
    protocol::{AreaAttackRequest, AreaAttackUpdate},
    puppet::{PuppetCursor, PuppetCursorBundle, Remote},
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
    puppets: Query<&Remote, With<PuppetCursor>>,
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
                    if !puppets.iter().any(|puppet| *puppet == *player) {
                        // counts both flags and known mines
                        let marked_count = field
                            .neighbor_cells(position)
                            .filter(|tile| matches!(tile, ClientTile::Flag | ClientTile::Mine))
                            .count() as u8;

                        if marked_count == *num_neighbors {
                            for (position, tile) in field.neighbors(position) {
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
    mut scene_spawner: ResMut<SceneSpawner>,
    mut commands: Commands,
    textures: Res<Textures>,
    old_scenes: Query<&SceneInstance, Or<(With<ClientTile>, With<Cursor>)>>,
    old_minefields: Query<Entity, With<Minefield>>,
) {
    for ev in events.iter() {
        if let AreaAttackUpdate::FieldShape(template) = ev {
            // despawn previously constructed entities
            for scn in &old_scenes {
                scene_spawner.despawn_instance(**scn);
            }
            for ent in &old_minefields {
                commands.entity(ent).despawn();
            }

            // spawn all received tiles
            let field = Minefield::new_shaped(
                |&position| {
                    commands
                        .spawn(ClientTileBundle {
                            tile: ClientTile::Unknown,
                            position,
                            scene: SceneBundle {
                                scene: textures.tile_empty.clone(),
                                transform: Transform::from_translation(
                                    position.absolute(TILE_SIZE, TILE_SIZE).extend_xz(0.0),
                                ),
                                ..default()
                            },
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
    mut puppets: Query<(&mut PuppetCursor, &mut Position, &Remote, Entity)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
            let mat = materials.add(StandardMaterial {
                emissive: (*color).into(),
                ..default()
            });
            if let Some((mut puppet, mut pos, _, mesh_id)) = puppets
                .iter_mut()
                .find(|(_, _, &Remote(remote), _)| remote == *id)
            {
                commands.entity(mesh_id).insert(NeedsMaterial(mat.clone()));
                puppet.0 = mat;
                *pos = *position;
            } else {
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
                        cursor: PuppetCursor(mat.clone()),
                        position: *position,
                        scene: SceneBundle {
                            scene: textures.cursor.clone(),
                            ..default()
                        },
                        remote: Remote(*id),
                    })
                    .insert(NeedsMaterial(mat))
                    .add_child(name);
            }
        }
    }
}

pub fn self_update(
    mut events: EventReader<AreaAttackUpdate>,
    mut commands: Commands,
    mut camera: Query<&mut Transform, With<Camera>>,
    textures: Res<Textures>,
    field: Query<Entity, With<Minefield>>,
    mut save_event: Local<Option<AreaAttackUpdate>>,
    mut assets: ResMut<Assets<StandardMaterial>>,
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
            let translation = position.absolute(TILE_SIZE, TILE_SIZE).extend_xz(0.0);
            camera.single_mut().tap_mut(|t| {
                t.translation.x = translation.x;
                t.translation.z = translation.z;
            });
            commands
                .spawn(CursorBundle {
                    cursor: Cursor::new(color.into(), field),
                    position,
                    texture: SceneBundle {
                        scene: textures.cursor.clone(),
                        transform: Transform::from_translation(translation),
                        ..default()
                    },
                })
                .insert(NeedsMaterial(assets.add(StandardMaterial {
                    emissive: color.into(),
                    ..default()
                })));
        } else {
            *save_event = Some(AreaAttackUpdate::SelfChange { color, position })
        }
    }
}

pub fn puppet_control(
    mut events: EventReader<AreaAttackUpdate>,
    mut puppets: Query<(&mut Position, &Remote), With<PuppetCursor>>,
    mut field: MinefieldQuery<&mut ClientTile>,
) {
    if field.get_single().is_none() {
        return;
    }
    for ev in events.iter() {
        match ev {
            AreaAttackUpdate::Reposition { id, position } => {
                *(puppets
                    .iter_mut()
                    .find_map(|(pos_mut, &Remote(rem))| (rem == *id).then_some(pos_mut))
                    .unwrap()) = *position;
            }
            AreaAttackUpdate::TileChanged { position, to } => {
                *field.get_single().unwrap().get_mut(*position).unwrap() = *to
            }
            _ => (),
        }
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
    mut commands: Commands,
    mut updated_tiles: Query<
        (&mut Handle<Scene>, &ClientTile, Entity),
        Or<(Added<ClientTile>, Changed<ClientTile>)>,
    >,
    textures: Res<Textures>,
    own_material: Res<MainCursorMaterial>,
    puppets: Query<(&PuppetCursor, &Remote)>,
    gltf: Res<Assets<Gltf>>,
) {
    updated_tiles.for_each_mut(|(mut sprite, state, tile_id)| {
        let mut tile = commands.entity(tile_id);
        *sprite = match state {
            ClientTile::Unknown => textures.tile_empty.clone(),
            ClientTile::Owned {
                player,
                num_neighbors,
            } => {
                tile.insert(NeedsMaterial(
                    puppets
                        .iter()
                        .find_map(|(PuppetCursor(color), &Remote(rem))| {
                            (rem == *player).then_some(color.clone())
                        })
                        .unwrap_or_else(|| own_material.clone()),
                ));
                gltf.get(&textures.mines_3d).unwrap().named_scenes
                    [&format!("f.tile_filled.{num_neighbors}")]
                    .clone()
            }
            // ClientTile::Mine => TextureAtlasSprite::new(11).tap_mut(|s| s.color = Color::default()),
            ClientTile::Flag => {
                tile.insert(NeedsMaterial(own_material.clone()));
                textures.tile_flagged.clone()
            }
            // ClientTile::Destroyed => TextureAtlasSprite::new(9).tap_mut(|s| s.color = Color::BLACK),
            _ => textures.tile_empty.clone(), // TODO fill in meshes for destroyed and mine tile
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
