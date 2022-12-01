use bevy::prelude::*;

#[derive(PartialEq, Eq, Component)]
pub enum AreaAttackState {
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

impl Default for AreaAttackState {
    fn default() -> Self {
        Self::Selecting
    }
}
