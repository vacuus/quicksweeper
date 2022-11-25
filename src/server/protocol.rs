use serde::{Deserialize, Serialize};
use bevy::prelude::*;

use crate::server::{GameDescriptor, GameMarker};

#[derive(Serialize, Deserialize, Debug)]
pub struct ActiveGame {
    marker: GameMarker,
    descriptor: GameDescriptor,
    players: Vec<String>,
}

pub struct ClientMessage {
    pub sender: Entity,
    pub data: ClientData,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientData {
    Greet { username: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    ActiveGames(Vec<ActiveGame>),
    Malformed,
}
