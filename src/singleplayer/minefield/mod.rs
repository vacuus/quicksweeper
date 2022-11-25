use bevy::prelude::*;

mod field;
mod load;
pub mod specific;
pub mod systems;

pub use field::*;
pub use load::BlankField;

pub enum GameOutcome {
    Failed,
    Succeeded,
}

pub struct MinefieldPlugin;

impl Plugin for MinefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<load::BlankField>()
            .add_asset_loader(load::FieldLoader)
            .add_event::<GameOutcome>()
            .add_system(specific::render_mines);
    }
}
