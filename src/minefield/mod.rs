use bevy::prelude::*;

mod field;
mod load;
pub mod query;
pub mod specific;
pub mod systems;

pub use field::*;
pub use load::FieldShape;

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
            .add_system(specific::render_mines);
    }
}
