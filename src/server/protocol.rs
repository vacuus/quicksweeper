use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::server::{GameDescriptor, GameMarker};

#[derive(Serialize, Deserialize, Debug)]
pub struct ActiveGame {
    pub marker: GameMarker,
    pub descriptor: GameDescriptor,
    pub players: Vec<String>,
}

pub struct ClientMessage {
    pub sender: Entity,
    pub data: ClientData,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientData {
    Greet { username: String },
    Create { game: GameMarker , data: Vec<u8>},
    Join {game: Entity},
    Ingame { data: Vec<u8> },
    ForceLeave,
    Games,
}

pub struct ServerMessage {
    pub receiver: Entity,
    pub data: ServerData,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerData {
    ActiveGames(Vec<ActiveGame>),
    Confirmed,
    Malformed,
}

#[derive(Debug)]
pub enum IngameEvent {
    Data {
        client: Entity,
        game: Entity,
        data: Vec<u8>,
    },
    Create {
        client: Entity,
        game: Entity,
        kind: GameMarker,
        data: Vec<u8>,
    },
}
