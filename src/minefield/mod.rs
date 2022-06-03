use crate::AppState;
use bevy::prelude::*;
use iyes_loopless::prelude::*;

mod field;
mod systems;

pub use field::*;
use systems::*;

pub struct MinefieldPlugin;

impl Plugin for MinefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(create_minefield.run_in_state(AppState::Loading))
            .add_system(generate_minefield.run_in_state(AppState::PreGame))
            .add_system(flag_cell.run_in_state(AppState::Game))
            .add_system(reveal_cell.run_in_state(AppState::Game))
            .add_system(
                field::render_mines.run_if(|state: Res<CurrentState<AppState>>| {
                    [AppState::PreGame, AppState::Game].contains(&state.0)
                }),
            )
            .add_enter_system(AppState::GameFailed, field::display_mines);
    }
}
