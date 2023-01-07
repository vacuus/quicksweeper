use std::collections::VecDeque;

use bevy::{hierarchy::HierarchyEvent, prelude::*};
use itertools::Itertools;
use rand::prelude::*;
use strum::IntoEnumIterator;

use crate::{
    common::{Contains, Position},
    load::Field,
    minefield::{query::MinefieldQuery, FieldShape, Minefield},
    server::{
        Access, Connection, ConnectionInfo, ConnectionSwitch, GameMarker, IngameEvent, LocalEvent,
    },
};

use super::{
    components::{
        AreaAttackBundle, ClientTile, Frozen, InitialSelections, Owner, PlayerBundle, PlayerColor,
        RevealTile, ServerTile, FREEZE_TIME,
    },
    protocol::{AreaAttackRequest, AreaAttackUpdate},
    states::AreaAttack,
    AreaAttackServer, AREA_ATTACK_MARKER,
};

#[derive(Component)]
pub struct Host;

pub fn create_game(
    mut commands: Commands,
    new_games: Query<(Entity, &GameMarker, &Children), Added<GameMarker>>,
    field_templates: Res<Assets<FieldShape>>,
    template_handles: Res<Field>,
) {
    for (game, kind, children) in new_games.iter() {
        if *kind != AREA_ATTACK_MARKER {
            continue;
        }

        // initialize game variables
        let template = template_handles.take_one(&mut rand::thread_rng()).clone();
        let bundle = AreaAttackBundle::new(&mut commands, game, template, &field_templates);
        commands.entity(game).insert(bundle);

        // there should only be one player right now, mark it as host
        let &player = children.iter().exactly_one().unwrap();
        commands.entity(player).insert(Host);
    }
}

pub fn unmark_init_access(mut access: Query<&mut Access, Added<AreaAttackServer>>) {
    access.for_each_mut(|mut access| {
        *access = Access::Open;
    })
}

// transcribes incoming messages to area attack events
pub fn net_events(
    mut network: EventReader<IngameEvent>,
    mut local: EventWriter<LocalEvent<AreaAttackRequest>>,
    games: Query<Entity, With<AreaAttackServer>>,
) {
    for ige @ IngameEvent { game, .. } in network.iter() {
        if games.contains(*game) {
            if let Ok(msg) = ige.transcribe() {
                local.send(msg);
            }
        }
    }
}

pub fn selection_transition(
    mut ev: EventReader<LocalEvent<AreaAttackRequest>>,
    mut games: Query<(Entity, &mut AreaAttack, &InitialSelections, &mut Access)>,
    mut minefields: MinefieldQuery<&mut ServerTile>,
    maybe_host: Query<(), With<Host>>,
    mut connections: Query<&mut Connection>,
    mut request_tile: EventWriter<RevealTile>,
) {
    for ev in ev.iter() {
        if !matches!(ev.data, AreaAttackRequest::StartGame) {
            continue;
        }
        if let Ok((game_id, mut state, selections, mut access)) = games.get_mut(ev.game) {
            if maybe_host.get(ev.player).is_ok() && matches!(*state, AreaAttack::Selecting) {
                *state = AreaAttack::Attack;

                let mut ignore = Vec::with_capacity(selections.len() * 16);

                // generate minefield while ignoring selected tiles
                for selection in selections.values() {
                    ignore.extend(selection.local_group());
                }

                // rng initialized before field in order to respect lifetime rules
                let mut rng = rand::thread_rng();
                let mut field = minefields.get(game_id).unwrap();

                field.choose_multiple(&ignore, &mut rng, |_, mut tile| {
                    *tile = ServerTile::Mine;
                });

                for (owner, selection) in selections.iter() {
                    request_tile.send(RevealTile {
                        position: *selection,
                        player: *owner,
                        game: ev.game,
                    })
                }

                connections.for_each_mut(|mut conn| {
                    conn.repeat_send_unchecked(AreaAttackUpdate::Transition(AreaAttack::Stage1));
                });

                // close joins
                *access = Access::Ingame;

                println!("Transitioned game to stage 1")
            }
        }
    }
}

pub fn reveal_tiles(
    mut requested: EventReader<RevealTile>,
    mut fields: MinefieldQuery<&mut ServerTile>,
    state: Query<&AreaAttack>,
    time: Res<Time>,
    mut players: Query<(&mut Frozen, &mut Connection)>,
    mut request_buffer: Local<VecDeque<RevealTile>>,
) {
    for &ev in requested.iter() {
        request_buffer.push_front(ev)
    }

    while let Some(RevealTile {
        position,
        player,
        game,
    }) = request_buffer.pop_front()
    {
        let Some(mut field) = fields.get(game) else { continue; };
        let state = state.get(game).unwrap();

        let (mut frozen, mut connection) = players.get_mut(player).unwrap();
        if frozen.is_some() {
            continue;
        }

        let Some(mut tile) = field.get_mut(position) else {continue;};
        match *tile {
            ServerTile::Empty => {
                *tile = ServerTile::Owned { player };
                let mine_count = field
                    .neighbor_cells(position)
                    .filter(|tile| matches!(tile, ServerTile::Mine | ServerTile::HardMine))
                    .count() as u8;

                if mine_count == 0 {
                    request_buffer.extend(field.neighbor_positions(position).map(|position| {
                        RevealTile {
                            position,
                            player,
                            game,
                        }
                    }))
                }
            }
            ServerTile::Mine => match state {
                AreaAttack::Stage1 => {
                    *tile = ServerTile::HardMine;
                    **frozen = Some(time.elapsed());
                    connection.send_logged(AreaAttackUpdate::Freeze);
                }
                AreaAttack::Attack => {
                    let mut rng = rand::thread_rng();
                    for p in position.radius(5) {
                        if let Some(mut tile) = field.get_mut(p) {
                            if !matches!(*tile, ServerTile::Destroyed) {
                                *tile = if rng.gen_bool(0.2) {
                                    ServerTile::Mine
                                } else {
                                    ServerTile::Empty
                                };
                            }
                        }
                    }
                    for p in position.disk_neighbors(5) {
                        if let Some(mut tile) = field.get_mut(p) {
                            if let ServerTile::Owned { player } = *tile {
                                *tile = ServerTile::Empty;
                                request_buffer.push_back(RevealTile {
                                    position: p,
                                    player,
                                    game,
                                })
                            }
                        }
                    }
                    *field.get_mut(position).unwrap() = ServerTile::Destroyed;
                }
                AreaAttack::Lock => todo!(),
                _ => (),
            },
            ServerTile::HardMine | ServerTile::Owned { .. } | ServerTile::Destroyed => {
                // do nothing
            }
        }
    }
}

pub fn unfreeze_players(time: Res<Time>, mut freeze: Query<&mut Frozen>) {
    let instant = time.elapsed();
    for mut f in freeze.iter_mut() {
        if let Some(start) = **f {
            if (instant - start) > FREEZE_TIME {
                **f = None;
            }
        }
    }
}

pub fn send_tiles_sys(
    tiles: Query<(&ServerTile, &Position, &Owner), Changed<ServerTile>>,
    mut minefields: MinefieldQuery<&ServerTile>,
    peers: Query<&Children>,
    mut connections: Query<&mut Connection>,
) {
    for (&tile, &position, &owner) in tiles.iter() {
        let minefield = minefields.get(*owner).unwrap();
        for &player_id in (peers.get(*owner).unwrap()).iter() {
            let mut connection = connections.get_mut(player_id).unwrap();
            let out_tile = match tile {
                ServerTile::Empty | ServerTile::Mine => ClientTile::Unknown,
                ServerTile::Owned { player: owner } => ClientTile::Owned {
                    player: owner,
                    num_neighbors: if player_id == owner {
                        minefield
                            .neighbor_cells(position)
                            .filter(|tile| matches!(tile, ServerTile::Mine | ServerTile::HardMine))
                            .count() as u8
                    } else {
                        0
                    },
                },
                ServerTile::HardMine => ClientTile::Mine,
                ServerTile::Destroyed => ClientTile::Destroyed,
            };

            connection.send_logged(AreaAttackUpdate::TileChanged {
                position,
                to: out_tile,
            });
        }
    }
}

pub fn broadcast_positions(
    mut requests: EventReader<LocalEvent<AreaAttackRequest>>,
    mut connections: Query<(&mut Position, &mut Connection)>,
    q_game: Query<&Children, With<AreaAttackServer>>,
) {
    for LocalEvent { player, game, data } in requests.iter() {
        if let AreaAttackRequest::Position(pos) = data {
            let peers = q_game.get(*game).unwrap();
            peers
                .iter()
                .filter(|&&e| *player != e)
                .for_each(|&conn_id| {
                    let (mut player_pos, mut sock) = connections.get_mut(conn_id).unwrap();
                    *player_pos = *pos;
                    sock.send_logged(AreaAttackUpdate::Reposition {
                        id: *player,
                        position: *pos,
                    });
                })
        }
    }
}

pub fn update_selecting_tile(
    mut requests: EventReader<LocalEvent<AreaAttackRequest>>,
    mut games: Query<(&AreaAttack, &mut InitialSelections, &Children)>,
    mut players: Query<(Entity, &mut Connection)>,
) {
    'event_loop: for LocalEvent { player, game, data } in requests.iter() {
        if let AreaAttackRequest::Reveal(requested) = data {
            let Ok((AreaAttack::Selecting, mut selections, children)) = games.get_mut(*game) else { continue; };

            for selection in selections
                .iter()
                .filter_map(|(owner, pos)| (owner != player).then_some(pos))
            {
                if selection.distance(requested) < 10.0 {
                    // TODO Notify client or have client indicate this itself
                    continue 'event_loop;
                }
            }

            if let Some(previous_position) = selections.remove(player) {
                for mut conn in players
                    .iter_mut()
                    .filter_map(|(e, it)| (children.contains(&e)).then_some(it))
                {
                    conn.send_logged(AreaAttackUpdate::TileChanged {
                        position: previous_position,
                        to: ClientTile::Unknown,
                    });
                }
            }
            for mut conn in players
                .iter_mut()
                .filter_map(|(e, it)| (children.contains(&e)).then_some(it))
            {
                conn.send_logged(AreaAttackUpdate::TileChanged {
                    position: *requested,
                    to: ClientTile::Owned {
                        player: *player,
                        num_neighbors: 0,
                    },
                });
            }
            selections.insert(*player, *requested);
        }
    }
}

/// Tile update system for when the host has begun the game
pub fn update_tile_playing(
    mut requests: EventReader<LocalEvent<AreaAttackRequest>>,
    mut games: Query<&AreaAttack>,
    mut reveal: EventWriter<RevealTile>,
) {
    for LocalEvent { player, game, data } in requests.iter() {
        let AreaAttackRequest::Reveal(pos) = data else {continue;};
        if !matches!(
            games.get_mut(*game),
            Ok(AreaAttack::Stage1 | AreaAttack::Attack)
        ) {
            continue;
        };

        reveal.send(RevealTile {
            position: *pos,
            player: *player,
            game: *game,
        })
    }
}

pub fn prepare_player(
    mut commands: Commands,
    mut ev: EventReader<ConnectionSwitch>,
    mut games: Query<(&Children, &Minefield, &FieldShape, &mut Access), With<AreaAttackServer>>,
    players: Query<(&ConnectionInfo, &mut PlayerColor, &Position)>,
    partial_connection_info: Query<&ConnectionInfo>,
    mut connections: Query<&mut Connection>,
) {
    for ev in ev.iter() {
        let ConnectionSwitch(HierarchyEvent::ChildAdded { child: player, parent: game }) = ev else {
            // TODO add ChildMoved variant as well
            continue;
        };
        let Ok((peers, minefield, shape, mut access)) = games.get_mut(*game) else { continue; };
        let Ok(mut this_connection) = connections.get_mut(*player) else {continue; };

        let peers = peers.iter().filter(|e| **e != *player).collect_vec();

        // guard if there are too many players
        if peers.len() >= 4 {
            *access = Access::Full // TODO: Reset when connection drops
        }

        let mut taken_colors = Vec::new();

        // send the selected board
        this_connection.repeat_send_unchecked(AreaAttackUpdate::FieldShape(shape.clone()));

        // send list of players and player properties
        for &&peer_id in peers.iter() {
            if peer_id == *player {
                continue;
            }
            let (ConnectionInfo { username }, &color, &position) = players.get(peer_id).unwrap();
            taken_colors.push(color);
            this_connection.repeat_send_unchecked(AreaAttackUpdate::PlayerProperties {
                id: peer_id,
                username: username.clone(),
                color,
                position,
            });
        }

        let assigned_color = PlayerColor::iter()
            .find(|co| !taken_colors.contains(co))
            .unwrap();
        let assigned_position = shape
            .center()
            .unwrap_or_else(|| minefield.iter_positions().next().unwrap());

        let this_username = &partial_connection_info.get(*player).unwrap().username;
        // send this player's properties to peers
        for &peer_id in peers {
            connections.get_mut(peer_id).unwrap().repeat_send_unchecked(
                AreaAttackUpdate::PlayerProperties {
                    id: *player,
                    username: this_username.clone(),
                    color: assigned_color,
                    position: assigned_position,
                },
            );
        }

        // reobtain this connection to respect the lifetime of the query (since the previous loop
        // reuses this query)
        let mut this_connection = connections.get_mut(*player).unwrap();

        // initialize some player properties
        commands.entity(*player).insert(PlayerBundle {
            color: assigned_color,
            position: assigned_position,
            frozen: Frozen::default(),
        });
        this_connection.repeat_send_unchecked(AreaAttackUpdate::SelfChange {
            color: assigned_color,
            position: assigned_position,
        });
    }
}
