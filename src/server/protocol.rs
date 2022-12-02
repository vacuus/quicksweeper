use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::server::GameMarker;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActiveGame {
    pub marker: GameMarker,
    pub id: Entity,
    pub players: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Greet { username: String },
    Create { game: GameMarker, data: Vec<u8> },
    Join { game: Entity },
    Ingame { data: Vec<u8> },
    ForceLeave,
    Games,
    GameTypes,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    ActiveGames(Vec<ActiveGame>),
    AvailableGames(Vec<GameMarker>),
    Malformed,
}

#[derive(Debug)]
pub enum IngameEvent {
    Data {
        player: Entity,
        game: Entity,
        data: Vec<u8>,
    },
    Create {
        player: Entity,
        game: Entity,
        kind: GameMarker,
        data: Vec<u8>,
    },
    /// A join event will also be issued when a player is auto-joined into a game upon creating it 
    Join {
        player: Entity,
        game: Entity,
    }
}
