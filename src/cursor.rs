use crate::common::{CheckCell, Direction, FlagCell, InitCheckCell, Position};
use crate::minefield::Minefield;
use bevy::{prelude::*, render::camera::Camera};
use gridly::prelude::*;
use tap::Tap;

#[derive(Debug)]
struct KeyTimers {
    key_left: HoldTimer,
    key_right: HoldTimer,
    key_up: HoldTimer,
    key_down: HoldTimer,
}

impl KeyTimers {
    fn with_bindings(activate_duration: f32, hold_delay: f32, bindings: &Bindings) -> Self {
        Self {
            key_left: HoldTimer::init(bindings.left, activate_duration, hold_delay),
            key_right: HoldTimer::init(bindings.right, activate_duration, hold_delay),
            key_up: HoldTimer::init(bindings.up, activate_duration, hold_delay),
            key_down: HoldTimer::init(bindings.down, activate_duration, hold_delay),
        }
    }
}

#[derive(Clone, Debug)]
struct Activations {
    left: bool,
    right: bool,
    up: bool,
    down: bool,
}

impl TryFrom<Activations> for Direction {
    type Error = ();

    fn try_from(value: Activations) -> Result<Self, Self::Error> {
        if value.up && value.down
            || value.left && value.right
            || !(value.up || value.down || value.left || value.right)
        {
            Err(())
        } else {
            Ok(if value.up {
                if value.left {
                    Direction::NorthWest
                } else if value.right {
                    Direction::NorthEast
                } else {
                    Direction::North
                }
            } else if value.down {
                if value.left {
                    Direction::SouthWest
                } else if value.right {
                    Direction::SouthEast
                } else {
                    Direction::South
                }
            } else if value.left {
                Direction::West
            } else {
                Direction::East
            })
        }
    }
}

impl Default for KeyTimers {
    fn default() -> Self {
        Self::init(0.25, 0.05)
    }
}

#[derive(Debug)]
struct HoldTimer {
    key: KeyCode,
    activate_timer: Timer,
    hold_timer: Timer,
}

impl HoldTimer {
    fn init(key: KeyCode, activate_duration: f32, hold_delay: f32) -> Self {
        Self {
            key,
            activate_timer: Timer::from_seconds(activate_duration, TimerMode::Once).tap_mut(|x| x.pause()),
            hold_timer: Timer::from_seconds(hold_delay, TimerMode::Repeating),
        }
    }

    fn tick(&mut self, time: &Res<Time>) {
        self.activate_timer.tick(time.delta());
        self.hold_timer.tick(time.delta());
    }

    fn tick_input(&mut self, time: &Res<Time>, input: &Res<Input<KeyCode>>) -> bool {
        self.tick(time);

        if input.just_released(self.key) {
            self.activate_timer.pause();
            self.activate_timer.reset();
        }

        let just_activated = input
            .just_pressed(self.key)
            .then(|| {
                self.activate_timer.reset();
                self.activate_timer.unpause();

                self.hold_timer.reset();
            })
            .is_some();

        self.activate_timer.finished() && self.hold_timer.just_finished() || just_activated
    }
}

impl KeyTimers {
    fn init(activate_duration: f32, hold_delay: f32) -> Self {
        Self {
            key_left: HoldTimer::init(KeyCode::A, activate_duration, hold_delay),
            key_right: HoldTimer::init(KeyCode::D, activate_duration, hold_delay),
            key_up: HoldTimer::init(KeyCode::W, activate_duration, hold_delay),
            key_down: HoldTimer::init(KeyCode::S, activate_duration, hold_delay),
        }
    }

    fn tick(&mut self, time: &Res<Time>) {
        self.key_left.tick(time);
        self.key_right.tick(time);
        self.key_up.tick(time);
        self.key_down.tick(time);
    }

    fn tick_input(&mut self, time: &Res<Time>, input: &Res<Input<KeyCode>>) -> Activations {
        self.tick(time);

        Activations {
            left: self.key_left.tick_input(time, input),
            right: self.key_right.tick_input(time, input),
            up: self.key_up.tick_input(time, input),
            down: self.key_down.tick_input(time, input),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    pub left: KeyCode,
    pub right: KeyCode,
    pub up: KeyCode,
    pub down: KeyCode,
    pub flag: KeyCode,
    pub check: KeyCode,
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            left: KeyCode::A,
            right: KeyCode::D,
            up: KeyCode::W,
            down: KeyCode::S,
            flag: KeyCode::F,
            check: KeyCode::Space,
        }
    }
}

#[derive(Component, Debug)]
pub struct Cursor {
    position: CursorPosition,
    timers: KeyTimers,
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
            timers: KeyTimers::with_bindings(0.25, 0.25, &bindings),
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

pub fn move_cursor(
    mut cursor: Query<&mut Cursor>,
    fields: Query<&Minefield>,
    kb: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for mut cursor in cursor.iter_mut() {
        let activated = cursor.timers.tick_input(&time, &kb);

        let Cursor {
            position: CursorPosition(position, minefield),
            ..
        } = *cursor;
        if let Some(next) = activated.try_into().ok().and_then(|direction| {
            position.neighbor_direction(direction).and_then(|neighbor| {
                let ent = fields.get(minefield).unwrap().get(&neighbor);
                matches!(ent, Ok(x) if x.is_some()).then(|| neighbor)
            })
        }) {
            cursor.position.0 = next;
        }
    }
}

pub fn translate_cursor(
    mut cursor: Query<(&mut Transform, &Cursor), Without<Camera>>,
    // mut camera: Query<&mut Transform, With<Camera>>,
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
        let target_translation = position.absolute(32.0, 32.0);
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
    let cursor_translation = cursor.get_single().unwrap().translation;
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
