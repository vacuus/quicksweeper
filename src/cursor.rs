use crate::{
    common::{CheckCell, Position},
    minefield::Minefield,
    AppState,
};
use bevy::{prelude::*, render::camera::Camera2d};
use derive_more::Deref;
use iyes_loopless::prelude::*;

#[derive(Deref)]
struct CursorTexture(Handle<Image>);

#[derive(Component, Debug)]
struct Cursor(Position);

fn load_cursor_texture(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tex: Handle<Image> = asset_server.load("cursor.png");
    commands.insert_resource(CursorTexture(tex));
}

fn create_cursor(mut commands: Commands, texture: Res<CursorTexture>) {
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
    mut cursor: Query<&mut Cursor>,
    kb: Res<Input<KeyCode>>,
    minefield: Query<&Minefield>,
) {
    let minefield = minefield.iter().next().unwrap();
    let max_x = minefield.num_columns() - 1;
    let max_y = minefield.num_rows() - 1;

    let mut cursor = cursor.iter_mut().next().unwrap(); // assume single cursor
    let Cursor(Position(ref mut x, ref mut y)) = *cursor;

    if kb.just_pressed(KeyCode::A) {
        *x = x.saturating_sub(1);
    } else if kb.just_pressed(KeyCode::D) && *x < max_x as u32 {
        *x += 1;
    }

    if kb.just_pressed(KeyCode::S) {
        *y = y.saturating_sub(1);
    } else if kb.just_pressed(KeyCode::W) && *y < max_y as u32 {
        *y += 1;
    }
}

fn translate_components(
    mut cursor: Query<(&mut Transform, &Cursor), Without<Camera2d>>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    mut previous: Local<Option<Position>>,
) {
    let (mut cursor_transform, Cursor(position)) = cursor.iter_mut().next().unwrap();
    let mut camera_transform = camera.iter_mut().next().unwrap();
    if previous.is_none() || previous.as_ref().unwrap() != position {
        let Position(x, y) = position;
        let translation = Vec3::new((*x * 32) as f32, (*y * 32) as f32, 3.0);

        cursor_transform.translation = translation.clone();
        camera_transform.translation = translation;

        *previous = Some(position.clone());
    }
}

fn check_cell(cursor: Query<&Cursor>, kb: Res<Input<KeyCode>>, mut ev: EventWriter<CheckCell>) {
    if kb.just_pressed(KeyCode::Space) {
        ev.send(CheckCell(cursor.get_single().unwrap().0.clone()));
    }
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_cursor_texture)
            .add_enter_system(AppState::Game, create_cursor)
            .add_system(move_cursor.run_in_state(AppState::Game))
            .add_system(translate_components.run_in_state(AppState::Game))
            .add_system(check_cell.run_in_state(AppState::Game));
    }
}
