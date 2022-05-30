use crate::{common::Position, minefield::Minefield, AppState};
use bevy::prelude::*;
use derive_more::Deref;
use iyes_loopless::prelude::*;

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

fn move_cursor(
    mut cursor: Query<(&mut Cursor, &mut Transform)>,
    kb: Res<Input<KeyCode>>,
    minefield: Res<Minefield>,
) {
    let max_x = minefield.num_columns() - 1;
    let max_y = minefield.num_rows() - 1;

    let (mut cursor, mut transform) = cursor.iter_mut().next().unwrap(); // assume single cursor
    let Cursor(Position(ref mut x, ref mut y)) = *cursor;

    if kb.just_pressed(KeyCode::A) {
        *x = x.saturating_sub(1);
    } else if kb.just_pressed(KeyCode::D) && *x < max_x {
        *x += 1;
    }

    if kb.just_pressed(KeyCode::S) {
        *y = y.saturating_sub(1);
    } else if kb.just_pressed(KeyCode::W) && *y < max_y {
        *y += 1;
    }

    transform.translation = Vec3::new((*x * 32) as f32, (*y * 32) as f32, 3.0);
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_cursor_texture)
            .add_enter_system(AppState::Game, create_cursor)
            .add_system(move_cursor.run_in_state(AppState::Game));
    }
}
