use bevy::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::server::GameMarker;

#[derive(Serialize, Deserialize, Debug)]
pub struct ActiveGame {
    pub marker: GameMarker,
    pub id: Entity,
    pub players: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Greeting {
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Create { game: GameMarker, args: Vec<u8> },
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

#[derive(Debug)]
pub struct LocalEvent<T> {
    pub player: Entity,
    pub game: Entity,
    pub data: T,
}

impl IngameEvent {
    pub fn transcribe<T>(&self) -> Result<LocalEvent<T>, rmp_serde::decode::Error>
    where
        T: DeserializeOwned,
    {
        Ok(LocalEvent {
            player: self.player,
            game: self.game,
            data: rmp_serde::from_slice(&self.data)?,
        })
    }
}
