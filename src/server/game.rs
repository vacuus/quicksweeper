//! ## How to create a quicksweeper game
//!
//! A quicksweeper gamemode is an entity that derives from the [GameBundle] bundle. When connections
//! are requested to it, the players will become children of the game, and the game will be given
//! management of their connections. Unfortunately, a gamemode right now is given trust over the
//! entire world, so caution should be exercised when modifying entities.
//!

use bevy::{prelude::*, utils::Uuid};

use serde::{Deserialize, Serialize};

use super::protocol::{ClientData, ClientMessage, ServerMessage, ServerData};

#[derive(Component, Serialize, Deserialize, Debug)]
pub struct GameDescriptor {
    pub name: String,
    pub description: String,
}

#[derive(Component, Deref, DerefMut, Serialize, Deserialize, Debug)]
pub struct GameMarker(pub Uuid);

#[derive(Component, Deref, DerefMut, Default)]
pub struct Players(pub Vec<Entity>);

#[derive(Bundle)]
pub struct GameBundle {
    pub marker: GameMarker,
    pub descriptor: GameDescriptor,
    pub players: Players,
}

pub fn top_level_connections(
    mut incoming: EventReader<ClientMessage>,
    mut outgoing: EventWriter<ServerMessage>,
    active_games: Query<(&GameMarker, &GameDescriptor, &Players)>,
) {
    let translate = |incoming: &ClientMessage| {
        let data = match incoming.data {
            ClientData::Greet { .. } => ServerData::Malformed,
        };

        ServerMessage {
            receiver: incoming.sender,
            data,
        }
    };

    for incoming in incoming.iter() {
        outgoing.send(translate(incoming))
    }
}
