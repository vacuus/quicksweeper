use crate::common::{CheckCell, FlagCell, InitCheckCell, Position};
use crate::singleplayer::minefield::specific::{MineCellState, CELL_SIZE};
use crate::singleplayer::minefield::Minefield;
use bevy::ecs::query::QueryEntityError;
use bevy::{prelude::*, render::camera::Camera};

#[derive(Debug, Clone, PartialEq, Eq)]
/// The entity field describes the minefield which it is placed on
pub struct CursorPosition(pub Position, pub Entity);

impl CursorPosition {
    pub fn iter_neighbors<'a>(
        &'a self,
        minefields: impl IntoIterator<Item = (Entity, &'a Minefield)>,
    ) -> Option<impl Iterator<Item = Self> + 'a> {
        minefields
            .into_iter()
            .find(|(ent, _)| *ent == self.1)
            .map(|(_, field)| {
                field
                    .iter_neighbor_positions(self.0)
                    .map(|pos| CursorPosition(pos, self.1))
            })
    }
}

pub struct Bindings {
    pub flag: KeyCode,
    pub check: KeyCode,
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            flag: KeyCode::F,
            check: KeyCode::Space,
        }
    }
}

#[derive(Component, Debug)]
pub struct Cursor {
    position: CursorPosition,
    check_key: KeyCode,
    flag_key: KeyCode,
}

impl Cursor {
    pub fn new(position: CursorPosition) -> Self {
        Self::new_with_keybindings(position, default())
    }

    pub fn new_with_keybindings(position: CursorPosition, bindings: Bindings) -> Self {
        Cursor {
            position,
            check_key: bindings.check,
            flag_key: bindings.flag,
        }
    }
}

pub fn destroy_cursors(mut commands: Commands, cursors: Query<Entity, With<Cursor>>) {
    cursors
        .iter()
        .for_each(|cursor| commands.entity(cursor).despawn())
}

/// Tracks the cursor to the system pointer
pub fn pointer_cursor(
    windows: Res<Windows>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut cursors: Query<&mut Cursor>,
    tiles: Query<&Transform>, // Does not include only tiles, but can be queried for tiles
    minefields: Query<&Minefield>,
) {
    let Ok(minefield) = minefields.get_single() else { return; };
    let Ok(root_tile) = tiles.get(minefield[&Position::ZERO]) else { return; };

    // code borrowed from bevy cheatbook
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = cameras.single();

    // get the window that the camera is displaying to (or the primary window)
    let wnd = windows.primary();

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = wnd.cursor_position() {
        // get the size of the window
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos = world_pos.truncate();

        // get the position of one corner of the board
        let field_transform = root_tile.translation.truncate();
        let offset = world_pos - field_transform + Vec2::splat(CELL_SIZE) / 2.;
        let pos = Position {
            x: (offset.x / CELL_SIZE).floor() as isize,
            y: (offset.y / CELL_SIZE).floor() as isize,
        };

        if minefield.is_contained(&pos) {
            cursors.single_mut().position.0 = pos;
        }
    }
}

pub fn translate_cursor(
    mut cursor: Query<(&mut Transform, &Cursor), Without<Camera>>,
    time: Res<Time>,
) {
    for (
        mut cursor_transform,
        Cursor {
            position: CursorPosition(position, _),
            ..
        },
    ) in cursor.iter_mut()
    {
        let cursor_translation = &mut cursor_transform.translation;

        // TODO: Use the offset of minefield to calculate `target_translation`
        let target_translation = position.absolute(CELL_SIZE, CELL_SIZE);
        let cursor_diff = target_translation - cursor_translation.truncate();

        // translate cursor
        if cursor_diff.length_squared() > 0.0001 {
            let scale = 10.0;
            *cursor_translation += (cursor_diff * time.delta_seconds() * scale).extend(0.0);
        } else {
            *cursor_translation = target_translation.extend(3.0);
        }
    }
}

/// Forces the camera to track a single cursor
pub fn track_cursor(
    cursor: Query<&Transform, (With<Cursor>, Without<Camera>)>,
    mut camera: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
) {
    if let Ok(&Transform {
        translation: cursor_translation,
        ..
    }) = cursor.get_single()
    {
        let camera_translation = &mut camera.single_mut().translation;
        let camera_diff = (cursor_translation - *camera_translation).truncate();

        // tranlate camera
        const MAX_CURSOR_TRAVEL: f32 = ((32 * 8) as u32).pow(2) as f32;
        let transform_magnitude = camera_diff.length_squared() - MAX_CURSOR_TRAVEL;
        if transform_magnitude > 0.0 {
            let scale = 0.4;
            *camera_translation += (camera_diff * time.delta_seconds() * scale).extend(0.0);
        }
    }
}

pub fn check_cell(
    cursor: Query<&Cursor>,
    kb: Res<Input<KeyCode>>,
    mut check: EventWriter<CheckCell>,
    mut flag: EventWriter<FlagCell>,
) {
    for Cursor {
        position,
        check_key,
        flag_key,
        ..
    } in cursor.iter()
    {
        if kb.just_pressed(*check_key) {
            println!("checking cell {position:?}");
            check.send(CheckCell(position.clone()));
        } else if kb.just_pressed(*flag_key) {
            flag.send(FlagCell(position.clone()));
        }
    }
}

pub fn init_check_cell(
    cursors: Query<&Cursor>,
    kb: Res<Input<KeyCode>>,
    mut ev: EventWriter<InitCheckCell>,
    fields: Query<(Entity, &Minefield)>,
) {
    if kb.just_pressed(KeyCode::Space) {
        println!("sending init check event");
        for Cursor {
            position: cursor_position @ CursorPosition(_, minefield),
            ..
        } in cursors.iter()
        {
            ev.send(InitCheckCell {
                minefield: *minefield,
                positions: cursor_position
                    .iter_neighbors(fields.iter())
                    .unwrap()
                    .map(|CursorPosition(pos, _)| pos)
                    .chain(std::iter::once(cursor_position.0))
                    .collect(),
            })
        }
    }
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(track_cursor)
            .add_system(translate_cursor)
            .add_system(pointer_cursor);
    }
}
