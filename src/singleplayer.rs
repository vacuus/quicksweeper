use crate::{
    common::InitCheckCell,
    minefield::{systems::*, GameOutcome},
};
use bevy::prelude::*;
use iyes_loopless::{
    prelude::{AppLooplessStateExt, IntoConditionalSystem},
    state::NextState,
};

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum SingleplayerState {
    Inactive,
    Loading,
    PreGame,
    Game,
    GameFailed,
    GameSuccess,
}

fn advance_to_end(mut commands: Commands, mut game_outcome: EventReader<GameOutcome>) {
    if let Some(outcome) = game_outcome.iter().next() {
        match outcome {
            GameOutcome::Failed => {
                commands.insert_resource(NextState(SingleplayerState::GameFailed))
            }
            GameOutcome::Succeeded => {
                commands.insert_resource(NextState(SingleplayerState::GameSuccess))
            }
        }
    }
}

fn advance_to_game(mut commands: Commands, mut init_move: EventReader<InitCheckCell>) {
    if let Some(_) = init_move.iter().next() {
        commands.insert_resource(NextState(SingleplayerState::Game))
    }
}

pub struct SingleplayerMode;

impl Plugin for SingleplayerMode {
    fn build(&self, app: &mut App) {
        app
            // state
            .add_loopless_state(SingleplayerState::Loading)
            // state change startup and cleanup
            .add_enter_system(SingleplayerState::PreGame, create_minefield)
            .add_exit_system(SingleplayerState::GameFailed, destroy_minefields)
            .add_exit_system(SingleplayerState::GameSuccess, destroy_minefields)
            .add_system(advance_to_game.run_in_state(SingleplayerState::PreGame))
            .add_system(advance_to_end.run_in_state(SingleplayerState::Game))
            .add_enter_system(SingleplayerState::GameFailed, display_mines)
            // in-game logic
            .add_system(generate_minefield.run_in_state(SingleplayerState::PreGame))
            .add_system(flag_cell.run_in_state(SingleplayerState::Game))
            .add_system(reveal_cell.run_in_state(SingleplayerState::Game));
    }
}
