use bevy::prelude::*;

use crate::minefield::Position;

struct Cursor(Position);

struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Cursor(Position(0, 0)));
    }
}