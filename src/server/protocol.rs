use serde::{Deserialize, Serialize};
use bevy::prelude::*;

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
    Games
}

pub struct ServerMessage {
    pub receiver: Entity,
    pub data: ServerData,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerData {
    ActiveGames(Vec<ActiveGame>),
    Malformed,
}
