use crate::common::{CheckCell, Direction, FlagCell, InitCheckCell, Position};
use crate::minefield::Minefield;
use bevy::{prelude::*, render::camera::Camera2d};
use tap::Tap;

#[derive(Debug)]
struct KeyTimers {
    key_left: HoldTimer,
    key_right: HoldTimer,
    key_up: HoldTimer,
    key_down: HoldTimer,
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
            activate_timer: Timer::from_seconds(activate_duration, false).tap_mut(|x| x.pause()),
            hold_timer: Timer::from_seconds(hold_delay, true),
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

#[derive(Debug, Clone)]
/// The entity field describes the minefield which it is placed on
pub struct CursorPosition(pub Position, pub Entity);

#[derive(Component, Debug)]
// pub struct Cursor(Position, Entity);
pub struct Cursor {
    position: CursorPosition,
    timers: KeyTimers,
}

impl Cursor {
    pub fn new(p: Position, e: Entity) -> Self {
        Cursor {
            position: CursorPosition(p, e),
            timers: KeyTimers::default(),
        }
    }
}

pub fn move_cursor(
    mut cursor: Query<&mut Cursor>,
    fields: Query<&Minefield>,
    kb: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let mut cursor = cursor.single_mut(); // assume single cursor
    let activated = cursor.timers.tick_input(&time, &kb);

    let Cursor {
        position: CursorPosition(position, minefield),
        ..
    } = *cursor;
    if let Some(next) = activated.try_into().ok().and_then(|direction| {
        position.neighbor_direction(direction).and_then(|neighbor| {
            fields
                .get(minefield)
                .unwrap()
                .contains_key(&neighbor)
                .then(|| neighbor)
        })
    }) {
        cursor.position.0 = next;
    }
}

pub fn translate_components(
    mut cursor: Query<(&mut Transform, &Cursor), Without<Camera2d>>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    // TODO: Get offset from minefield
    let (
        mut cursor_transform,
        Cursor {
            position: CursorPosition(position, _),
            ..
        },
    ) = cursor.single_mut();
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

pub fn check_cell(
    cursor: Query<&Cursor>,
    kb: Res<Input<KeyCode>>,
    mut check: EventWriter<CheckCell>,
    mut flag: EventWriter<FlagCell>,
) {
    let Cursor { position, .. } = &cursor.get_single().unwrap();
    if kb.just_pressed(KeyCode::Space) {
        check.send(CheckCell(position.clone()));
    } else if kb.just_pressed(KeyCode::F) {
        flag.send(FlagCell(position.clone()));
    }
}

pub fn init_check_cell(
    cursor: Query<&Cursor>,
    kb: Res<Input<KeyCode>>,
    mut ev: EventWriter<InitCheckCell>,
) {
    if kb.just_pressed(KeyCode::Space) {
        let Cursor { position, .. } = cursor.single();
        ev.send(InitCheckCell(position.clone()));
    }
}
