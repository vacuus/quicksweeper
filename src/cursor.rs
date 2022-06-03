use crate::{
    common::{CheckCell, FlagCell, InitCheckCell, Position},
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
    time: Res<Time>,
) {
    let (mut cursor_transform, Cursor(position)) = cursor.single_mut();
    let cursor_translation = &mut cursor_transform.translation;
    let camera_translation = &mut camera.single_mut().translation;

    let target_translation = position.absolute(32.0, 32.0);
    let cursor_diff = target_translation - cursor_translation.truncate();
    let camera_diff = (*cursor_translation - *camera_translation).truncate();

    // tranlate camera
    const MAX_CURSOR_TRAVEL: f32 = ((32 * 8) as u32).pow(2) as f32;
    let transform_magnitude = camera_diff.length_squared() - MAX_CURSOR_TRAVEL;
    if transform_magnitude > 0.0 {
        let scale = 0.4;
        *camera_translation += (camera_diff * time.delta_seconds() * scale).extend(0.0);
    }

    // translate cursor
    if cursor_diff.length_squared() > 0.0001 {
        let scale = 10.0;
        *cursor_translation += (cursor_diff * time.delta_seconds() * scale).extend(0.0);
    } else {
        *cursor_translation = target_translation.extend(3.0);
    }
}

fn check_cell(
    cursor: Query<&Cursor>,
    kb: Res<Input<KeyCode>>,
    mut check: EventWriter<CheckCell>,
    mut flag: EventWriter<FlagCell>,
) {
    let pos = &cursor.get_single().unwrap().0;
    if kb.just_pressed(KeyCode::Space) {
        check.send(CheckCell(pos.clone()));
    } else if kb.just_pressed(KeyCode::F) {
        flag.send(FlagCell(pos.clone()));
    }
}

fn init_check_cell(
    cursor: Query<&Cursor>,
    kb: Res<Input<KeyCode>>,
    mut ev: EventWriter<InitCheckCell>,
) {
    if kb.just_pressed(KeyCode::Space) {
        ev.send(InitCheckCell(cursor.get_single().unwrap().0.clone()));
    }
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_cursor_texture)
            .add_enter_system(AppState::PreGame, create_cursor)
            .add_system_set(
                ConditionSet::new()
                    .run_if(|state: Res<CurrentState<AppState>>| {
                        [AppState::PreGame, AppState::Game].contains(&state.0)
                    })
                    .with_system(move_cursor)
                    .with_system(translate_components)
                    .into(),
            )
            .add_system(init_check_cell.run_in_state(AppState::PreGame))
            .add_system(check_cell.run_in_state(AppState::Game));
    }
}
