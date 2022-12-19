use crate::area_attack::puppet::PuppetCursor;
use crate::common::{CheckCell, FlagCell, InitCheckCell, Position};
use crate::singleplayer::minefield::specific::TILE_SIZE;
use crate::singleplayer::minefield::Minefield;
use bevy::input::mouse::MouseWheel;
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

#[derive(Bundle)]
pub struct CursorBundle {
    pub cursor: Cursor,
    pub position: Position,
    pub texture: SpriteBundle,
}

#[derive(Component, Debug)]
pub struct Cursor {
    pub color: Color,
    pub owning_minefield: Entity,
    pub check_key: KeyCode,
    pub flag_key: KeyCode,
}

impl Cursor {
    pub fn new(color: Color, owning_minefield: Entity) -> Self {
        Self::new_with_keybindings(color, owning_minefield, default())
    }

    pub fn new_with_keybindings(
        color: Color,
        owning_minefield: Entity,
        bindings: Bindings,
    ) -> Self {
        Cursor {
            color,
            owning_minefield,
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
    mut cursors: Query<(&Cursor, &mut Position)>,
    tiles: Query<&Transform>, // Does not include only tiles, but can be queried for tiles
    minefields: Query<&Minefield>,
) {
    let Ok((cursor, mut position)) = cursors.get_single_mut() else { return; };
    let Ok(minefield) = minefields.get(cursor.owning_minefield) else { return; };
    let open_position = minefield.iter_positions().next().unwrap();
    let Ok(root_tile) = tiles.get(minefield[&open_position]) else { return; };

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

        // get the position relative to one tile on the board
        let field_transform =
            root_tile.translation.truncate() - open_position.absolute(TILE_SIZE, TILE_SIZE);
        let offset = world_pos - field_transform + Vec2::splat(TILE_SIZE) / 2.;
        let pos = Position {
            x: (offset.x / TILE_SIZE).floor() as isize,
            y: (offset.y / TILE_SIZE).floor() as isize,
        };

        if minefield.is_contained(&pos) && *position != pos {
            *position = pos;
        }
    }
}

pub fn translate_cursor(
    mut cursor: Query<(&mut Transform, &Position), Or<(With<Cursor>, With<PuppetCursor>)>>,
    time: Res<Time>,
) {
    for (mut cursor_transform, position) in cursor.iter_mut() {
        let cursor_translation = &mut cursor_transform.translation;

        // TODO: Use the offset of minefield to calculate `target_translation`
        let target_translation = position.absolute(TILE_SIZE, TILE_SIZE);
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
    cursor: Query<(&Cursor, &Position)>,
    kb: Res<Input<KeyCode>>,
    mut check: EventWriter<CheckCell>,
    mut flag: EventWriter<FlagCell>,
) {
    for (
        &Cursor {
            owning_minefield,
            check_key,
            flag_key,
            ..
        },
        &position,
    ) in cursor.iter()
    {
        if kb.just_pressed(check_key) {
            check.send(CheckCell(CursorPosition(position, owning_minefield)));
        } else if kb.just_pressed(flag_key) {
            flag.send(FlagCell(CursorPosition(position, owning_minefield)));
        }
    }
}

pub fn init_check_cell(
    cursors: Query<(&Cursor, &Position)>,
    kb: Res<Input<KeyCode>>,
    mut ev: EventWriter<InitCheckCell>,
    fields: Query<(Entity, &Minefield)>,
) {
    if kb.just_pressed(KeyCode::Space) {
        println!("sending init check event");
        for (
            &Cursor {
                // position: cursor_position @ CursorPosition(_, minefield),
                owning_minefield: minefield,
                ..
            },
            &position,
        ) in cursors.iter()
        {
            let cursor_position = CursorPosition(position, minefield);
            ev.send(InitCheckCell {
                minefield,
                positions: cursor_position
                    .iter_neighbors(fields.iter())
                    .unwrap()
                    .map(|CursorPosition(pos, _)| pos)
                    .chain(std::iter::once(position))
                    .collect(),
            })
        }
    }
}

fn zoom_camera(
    mut camera: Query<&mut OrthographicProjection, With<Camera2d>>,
    mut scroll: EventReader<MouseWheel>,
    mut scale: Local<f32>,
) {
    for scroll in scroll.iter() {
        *scale = (*scale + scroll.y).clamp(-3f32, 3f32);
        if let Ok(mut proj) = camera.get_single_mut() {
            proj.scale = 2f32.powf(*scale);
        }
    }
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(track_cursor)
            .add_system(translate_cursor)
            .add_system(zoom_camera)
            .add_system(pointer_cursor);
    }
}
