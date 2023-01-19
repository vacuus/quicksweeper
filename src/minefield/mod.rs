use bevy::prelude::*;

mod field;
mod load;
pub mod query;
pub mod specific;
pub mod systems;

pub use field::*;
use iyes_loopless::prelude::IntoConditionalSystem;
pub use load::FieldShape;

use crate::main_menu::Menu;

pub enum GameOutcome {
    Failed,
    Succeeded,
}

pub struct MinefieldPlugin;

impl Plugin for MinefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<load::FieldShape>()
            .add_asset_loader(load::FieldLoader)
            .add_event::<GameOutcome>()
            .add_system(specific::render_mines.run_not_in_state(Menu::Loading));
    }
}
