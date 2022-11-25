
use serde::{Serialize, Deserialize};

use crate::server::{GameDescriptor, GameMarker};

#[derive(Serialize, Deserialize, Debug)]
pub struct ActiveGame {
    marker: GameMarker,
    descriptor: GameDescriptor,
    players: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Greet {
        username: String,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    ActiveGames(Vec<ActiveGame>),
}