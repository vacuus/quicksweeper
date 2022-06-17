use crate::load::Field;
use crate::minefield::{BlankField, Minefield};
use crate::state::ConditionalHelpersExt;
use crate::{
    common::{CheckCell, Direction, FlagCell, InitCheckCell, Position},
    SingleplayerState,
};
use bevy::{prelude::*, render::camera::Camera2d};
use derive_more::Deref;
use iyes_loopless::prelude::*;
use tap::Tap;

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

#[derive(Deref)]
struct CursorTexture(Handle<Image>);

/// The entity field describes the minefield which it is placed on
#[derive(Component, Debug)]
struct Cursor(Position, Entity);

fn load_cursor_texture(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tex: Handle<Image> = asset_server.load("cursor.png");
    commands.insert_resource(CursorTexture(tex));
}

fn create_cursor(
    mut commands: Commands,
    texture: Res<CursorTexture>,
    field_template: Res<Field>,
    field_templates: Res<Assets<BlankField>>,
    fields: Query<Entity, Added<Minefield>>,
) {
    if !fields.is_empty() {
        #[allow(clippy::or_fun_call)]
        let init_position = field_templates
            .get(field_template.field.clone())
            .unwrap()
            .center()
            .unwrap_or(Position::new(0, 0));

        // create cursor
        commands
            .spawn_bundle(SpriteBundle {
                texture: (*texture).clone(),
                transform: Transform {
                    translation: init_position.absolute(32.0, 32.0).extend(3.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Cursor(init_position, fields.single()));
    }
}

fn move_cursor(
    mut cursor: Query<&mut Cursor>,
    fields: Query<&Minefield>,
    kb: Res<Input<KeyCode>>,
    mut key_timers: Local<KeyTimers>,
    time: Res<Time>,
) {
    let mut cursor = cursor.single_mut(); // assume single cursor
    let activated = key_timers.tick_input(&time, &kb);

    let Cursor(pos, ent) = *cursor;
    if let Some(next) = activated.try_into().ok().and_then(|direction| {
        pos.neighbor_direction(direction).and_then(|neighbor| {
            fields
                .get(ent)
                .unwrap()
                .contains_key(&neighbor)
                .then(|| neighbor)
        })
    }) {
        cursor.0 = next;
    }
}

fn destroy_cursor(mut commands: Commands, cursor: Query<Entity, With<Cursor>>) {
    commands.entity(cursor.single()).despawn();
}

fn translate_components(
    mut cursor: Query<(&mut Transform, &Cursor), Without<Camera2d>>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    let (mut cursor_transform, Cursor(position, _)) = cursor.single_mut();
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
    let Cursor(pos, ent) = &cursor.get_single().unwrap();
    if kb.just_pressed(KeyCode::Space) {
        check.send(CheckCell(*pos, *ent));
    } else if kb.just_pressed(KeyCode::F) {
        flag.send(FlagCell(*pos, *ent));
    }
}

fn init_check_cell(
    cursor: Query<&Cursor>,
    kb: Res<Input<KeyCode>>,
    mut ev: EventWriter<InitCheckCell>,
) {
    if kb.just_pressed(KeyCode::Space) {
        let Cursor(pos, ent) = cursor.single();
        ev.send(InitCheckCell(*pos, *ent));
    }
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_cursor_texture)
            .add_enter_system(SingleplayerState::Loading, |mut commands: Commands| {
                commands.spawn_bundle(OrthographicCameraBundle::new_2d());
            })
            .add_system(create_cursor.run_in_state(SingleplayerState::PreGame))
            .add_system(
                move_cursor
                    .into_conditional()
                    .run_in_states([SingleplayerState::PreGame, SingleplayerState::Game])
                    .run_if(|q: Query<Entity, With<Cursor>>| !q.is_empty()),
            )
            .add_system(
                translate_components
                    .into_conditional()
                    .run_in_states([
                        SingleplayerState::PreGame,
                        SingleplayerState::Game,
                        SingleplayerState::GameFailed,
                        SingleplayerState::GameSuccess,
                    ])
                    .run_if(|q: Query<Entity, With<Cursor>>| !q.is_empty()),
            )
            .add_exit_system(SingleplayerState::GameFailed, destroy_cursor)
            .add_exit_system(SingleplayerState::GameSuccess, destroy_cursor)
            .add_system(init_check_cell.run_in_state(SingleplayerState::PreGame))
            .add_system(check_cell.run_in_state(SingleplayerState::Game));
    }
}
