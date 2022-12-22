use bevy::prelude::*;

use serde::{Deserialize, Serialize};

use crate::{common::Position, minefield::FieldShape};

use super::{
    components::{ClientTile, PlayerColor},
    states::AreaAttackState,
};

#[derive(Serialize, Deserialize)]
pub enum AreaAttackUpdate {
    FieldShape(FieldShape),
    /// Can both be sent on the creation of a new player as well as when a player updates its
    /// properties, and will be sent in a batch to any player who joins the game
    PlayerProperties {
        id: Entity,
        username: String,
        color: PlayerColor,
        position: Position,
    },
    Reposition {
        id: Entity,
        position: Position,
    },
    /// Will be sent to the player if the game autosets its properties (e.g. on initial join)
    SelfChange {
        color: PlayerColor,
        position: Position,
    },
    TileChanged {
        position: Position,
        to: ClientTile,
    },
    Transition(AreaAttackState),
    /// Issued to a client when it attempts to join a full game
    Full,
    NotHost,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AreaAttackRequest {
    StartGame,
    Reveal(Position),
    Position(Position),
    Color(PlayerColor),
}
