use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Eq, Clone, Hash, Debug, Component, Serialize, Deserialize)]
pub enum AreaAttackState {
    /// Only for client -- to indicate that there is no Area Attack game in progress
    Inactive,
    /// Players are selecting the cells that they will begin on
    Selecting,
    /// The first stage of Area Attack, without restrictions
    Stage1,
    /// The attack stage, which follows [AreaAttackState::Stage1]
    Attack,
    /// The lock stage, which follows [AreaAttackState::Attack]
    Lock,
    /// After the game is done, rank is awarded (if it is valid for this game to do so) and players
    /// are prompted to rematch or exit
    Finishing,
}
