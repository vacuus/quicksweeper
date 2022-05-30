use crate::{minefield::Position, AppState};
use bevy::prelude::*;
use derive_more::Deref;
use iyes_loopless::prelude::AppLooplessStateExt;

#[derive(Deref)]
struct CursorTexture(Handle<Image>);

#[derive(Component)]
struct Cursor(Position);

fn load_cursor_texture(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tex: Handle<Image> = asset_server.load("cursor.png");
    commands.insert_resource(CursorTexture(tex));
}

fn create_cursor(mut commands: Commands, texture: Res<CursorTexture>) {
    println!("I rendered the thing");
    commands
        .spawn_bundle(SpriteBundle {
            texture: (*texture).clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 3.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cursor(Position(0, 0)));
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_cursor_texture)
            .add_enter_system(AppState::Game, create_cursor);
    }
}
