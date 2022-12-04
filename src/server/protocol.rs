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
    Create { game: GameMarker },
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
pub struct IngameEvent {
    pub player: Entity,
    pub game: Entity,
    pub data: Vec<u8>,
}
