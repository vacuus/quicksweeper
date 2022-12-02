use bevy::prelude::*;

use serde::{Serialize, Deserialize};

use crate::{singleplayer::minefield::FieldShape, common::Position};

use super::tile::ClientTile;

#[derive(Serialize, Deserialize)]
enum PlayerColor {
    Yellow,
    Green,
    Blue,
}

#[derive(Serialize, Deserialize)]
enum AreaAttackUpdate {
    FieldShape(FieldShape),
    /// Can both be sent on the creation of a new player as well as when a player updates its
    /// properties, and will be sent in a batch to any player who joins the game
    PlayerModified {
        id: Entity,
        color: PlayerColor,
    },
    TileChanged {
        position: Position,
        to: ClientTile, 
    }
}

#[derive(Serialize, Deserialize)]
enum AreaAttackRequest {
    Reveal(Position), 
    Color(PlayerColor),
}