use crate::SingleplayerState;
use bevy::prelude::*;
use iyes_loopless::prelude::*;

mod field;
mod load;
mod systems;

pub use field::*;
pub use load::BlankField;
use systems::*;

pub struct MinefieldPlugin;

impl Plugin for MinefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<load::BlankField>()
            .add_asset_loader(load::FieldLoader)
            .add_enter_system(SingleplayerState::PreGame, create_minefield)
            .add_exit_system(SingleplayerState::GameFailed, destroy_minefields)
            .add_exit_system(SingleplayerState::GameSuccess, destroy_minefields)
            .add_system(generate_minefield.run_in_state(SingleplayerState::PreGame))
            .add_system(flag_cell.run_in_state(SingleplayerState::Game))
            .add_system(reveal_cell.run_in_state(SingleplayerState::Game))
            .add_system(field::render_mines.run_if(
                |state: Res<CurrentState<SingleplayerState>>| {
                    [SingleplayerState::PreGame, SingleplayerState::Game].contains(&state.0)
                },
            ))
            .add_enter_system(SingleplayerState::GameFailed, field::display_mines);
    }
}
