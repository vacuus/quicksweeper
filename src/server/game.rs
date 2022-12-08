//! ## How to create a quicksweeper game
//!
//! A quicksweeper gamemode is an entity that derives from the [GameBundle] bundle. When connections
//! are requested to it, the players will become children of the game, and the game will be given
//! management of their connections. Unfortunately, a gamemode right now is given trust over the
//! entire world, so caution should be exercised when modifying entities.
//!

use bevy::{hierarchy::HierarchyEvent, prelude::*, utils::Uuid};
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

/// A component on a game describing whether or not a game is allowed to be connected to by the
/// player
#[derive(Component)]
pub enum Access {
    /// Each game is spawned with this access. It is up to the game to update this to reflect that
    /// it is ready to receive players (by changing to Open access)
    Initializing,
    Open,
    Full,
    Ingame,
}

#[derive(Bundle)]
pub struct GameBundle {
    pub marker: GameMarker,
    pub access: Access,
}

pub fn game_messages(
    mut commands: Commands,
    mut clients: Query<(Entity, &mut Connection, &Parent)>,
    mut game_events: EventWriter<IngameEvent>,
) {
    for (player, mut socket, game) in clients.iter_mut() {
        match socket.recv_message() {
            Some(Ok(ClientMessage::Ingame { data })) => game_events.send(IngameEvent {
                player,
                game: **game,
                data,
            }),
            Some(Ok(ClientMessage::ForceLeave)) => {
                commands.entity(**game).remove_children(&[player]);
            }
            Some(_) => {
                let _ = socket.send_message(ServerMessage::Malformed); // TODO report this later
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
    registry: Res<GameRegistry>,
) {
    for (player, mut socket) in clients.iter_mut() {
        match socket.recv_message() {
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

                let _ = socket.send_message(msg);
            }
            Some(Ok(ClientMessage::GameTypes)) => {
                let _ = socket.send_message(ServerMessage::AvailableGames(
                    registry.keys().copied().collect(),
                ));
            }
            Some(Ok(ClientMessage::Create { game, args: _args, })) => {  //TODO: Pass arguments down
                commands
                    .spawn(GameBundle {
                        marker: game,
                        access: Access::Initializing,
                    })
                    .add_child(player);
            }
            Some(Ok(ClientMessage::Join { game })) => {
                if let Some(mut ent) = commands.get_entity(game) {
                    ent.add_child(player);
                } else {
                    let _ = socket.send_message(ServerMessage::Malformed);
                }
            }
            Some(_) => {
                let _ = socket.send_message(ServerMessage::Malformed); // TODO report this later
            }
            _ => (),
        };
    }
}

fn access<'query>(
    event: &HierarchyEvent,
    parents: &'query Query<&Access>,
) -> Option<&'query Access> {
    use HierarchyEvent::*;
    let (ChildAdded { parent, .. }
    | ChildMoved {
        new_parent: parent, ..
    }
    | ChildRemoved { parent, .. }) = event;
    parents.get(*parent).ok()
}

#[derive(Debug)]
pub struct ConnectionSwitch(pub HierarchyEvent);

pub fn delay_hierarchy_events(
    mut hierarchy_events: EventReader<HierarchyEvent>,
    mut connection_events: EventWriter<ConnectionSwitch>,
    targets: Query<&Access>,
    mut store: Local<Vec<HierarchyEvent>>,
) {
    // TODO drain filter again...
    {
        let mut ix = 0;
        while ix < store.len() {
            if !matches!(access(&store[ix], &targets), Some(Access::Initializing)) {
                println!("Resending event {:?}", store[ix]);
                connection_events.send(ConnectionSwitch(store.swap_remove(ix)))
            } else {
                ix += 1;
            }
        }
    }

    for event in hierarchy_events.iter() {
        if matches!(access(event, &targets), Some(Access::Initializing)) {
            println!("Delaying event {event:?}");
            store.push(event.clone());
        } else if access(event, &targets).is_some() {
            // That is, the entity is a game at all
            connection_events.send(ConnectionSwitch(event.clone()))
        }
    }
}
