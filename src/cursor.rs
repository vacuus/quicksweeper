use bevy::prelude::*;

use crate::minefield::Position;

struct Cursor {
    pos: Position,
    tex: Handle<Image>,
}

fn build_cursor(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tex: Handle<Image> = asset_server.load("cursor.png");

    commands.insert_resource(Cursor {
        pos: Position(0, 0),
        tex,
    });
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(build_cursor);
    }
}
