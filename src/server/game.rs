//! ## How to create a quicksweeper game
//!
//! A quicksweeper gamemode is an entity that derives from the [GameBundle] bundle. When connections
//! are requested to it, the players will become children of the game, and the game will be given
//! management of their connections. Unfortunately, a gamemode right now is given trust over the
//! entire world, so caution should be exercised when modifying entities.
//!

use bevy::{prelude::*, utils::Uuid};
use serde::{Deserialize, Serialize};

use crate::registry::GameRegistry;

use super::{
    protocol::{ActiveGame, ClientMessage, ServerMessage},
    sockets::{Connection, ConnectionInfo},
    IngameEvent, MessageSocket,
};

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct GameDescriptor {
    pub name: String,
    pub description: String,
}

#[derive(
    Component, Deref, DerefMut, Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash,
)]
pub struct GameMarker(pub Uuid);

#[derive(Bundle)]
pub struct GameBundle {
    pub marker: GameMarker,
}

pub fn game_messages(
    mut commands: Commands,
    mut clients: Query<(Entity, &mut Connection, &Parent)>,
    mut game_events: EventWriter<IngameEvent>,
) {
    for (player, mut socket, game) in clients.iter_mut() {
        match socket.read_data() {
            Some(Ok(ClientMessage::Ingame { data })) => game_events.send(IngameEvent::Data {
                player,
                game: **game,
                data,
            }),
            Some(Ok(ClientMessage::ForceLeave)) => {
                commands.entity(**game).remove_children(&[player]);
            }
            Some(_) => {
                let _ = socket.write_data(ServerMessage::Malformed); // TODO report this later
            }
            None => (),
        }
    }
}

pub fn server_messages(
    mut commands: Commands,
    mut clients: Query<(Entity, &mut Connection), (With<ConnectionInfo>, Without<Parent>)>,
    q_players: Query<&ConnectionInfo>,
    active_games: Query<(Entity, &GameMarker, &Children)>,
    mut game_events: EventWriter<IngameEvent>,
    registry: Res<GameRegistry>,
) {
    for (player, mut socket) in clients.iter_mut() {
        match socket.read_data() {
            Some(Ok(ClientMessage::Games)) => {
                let msg = ServerMessage::ActiveGames(
                    active_games
                        .iter()
                        .map(|(id, &marker, player_ids)| {
                            let players = player_ids
                                .iter()
                                .map(|&ent| q_players.get(ent).unwrap().username.clone())
                                .collect();
                            ActiveGame {
                                marker,
                                id,
                                players,
                            }
                        })
                        .collect(),
                );

                let _ = socket.write_data(msg);
            }
            Some(Ok(ClientMessage::GameTypes)) => {
                let _ = socket.write_data(ServerMessage::AvailableGames(
                    registry.keys().copied().collect(),
                ));
            }
            Some(Ok(ClientMessage::Create { game, data })) => {
                let game_id = commands.spawn((game,)).add_child(player).id();
                game_events.send(IngameEvent::Create {
                    player,
                    game: game_id,
                    kind: game,
                    data,
                });
            }
            Some(Ok(ClientMessage::Join { game })) => {
                if let Some(mut ent) = commands.get_entity(game) {
                    ent.add_child(player);
                } else {
                    let _ = socket.write_data(ServerMessage::Malformed);
                }
            }
            Some(_) => {
                let _ = socket.write_data(ServerMessage::Malformed); // TODO report this later
            }
            _ => (),
        };
    }
}
