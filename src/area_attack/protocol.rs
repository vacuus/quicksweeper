use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{common::Position, singleplayer::minefield::FieldShape};

use super::components::{ClientTile, PlayerColor};

#[derive(Serialize, Deserialize)]
pub enum AreaAttackUpdate {
    FieldShape(FieldShape),
    /// Can both be sent on the creation of a new player as well as when a player updates its
    /// properties, and will be sent in a batch to any player who joins the game
    PlayerChange {
        id: Entity,
        username: Option<String>,
        color: Option<PlayerColor>,
        position: Option<Position>,
    },
    /// Will be sent to the player if the game autosets its properties (e.g. on initial join)
    SelfChange {
        color: PlayerColor,
    },
    TileChanged {
        position: Position,
        to: ClientTile,
    },
    /// Issued to a client when it attempts to join a full game
    Full,
}

#[derive(Serialize, Deserialize)]
pub enum AreaAttackRequest {
    Reveal(Position),
    Color(PlayerColor),
}
