use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use tap::Tap;

use crate::{
    common::Position,
    minefield::{FieldShape, Minefield},
};

use super::{states::AreaAttack, AreaAttackServer};

pub const FREEZE_DURATION: Duration = Duration::from_secs(5);

#[derive(Component)]
pub struct StageTimer{
    previous_time: Duration,
    timer: Timer,
}

impl StageTimer {
    pub fn tick(&mut self, delta: Duration) -> &Self{
        self.previous_time = self.timer.elapsed();
        self.timer.tick(delta);
        self
    }

    pub fn has_just_elapsed(&self, duration: Duration) -> bool {
        self.previous_time < duration && duration < self.timer.elapsed()
    }

    pub fn just_finished(&mut self) -> bool {
        self.timer.just_finished()
    }

    #[allow(dead_code)]
    pub fn pause(&mut self) {
        self.timer.pause()
    }

    pub fn unpause(&mut self) {
        self.timer.unpause()
    }
}

impl Default for StageTimer {
    fn default() -> Self {
        Self{
            previous_time: Duration::from_nanos(0),
            timer: Timer::new(Duration::from_secs(7 * 60), TimerMode::Once).tap_mut(|timer| timer.pause()),
        }
    }
}

#[derive(Bundle)]
pub struct AreaAttackBundle {
    field: Minefield,
    template: FieldShape,
    selections: InitialSelections,
    state: AreaAttack,
    typed_marker: AreaAttackServer,
    stage_timer: StageTimer,
}

impl AreaAttackBundle {
    pub fn new(
        commands: &mut Commands,
        owner: Entity,
        template: Handle<FieldShape>,
        template_set: &Res<Assets<FieldShape>>,
    ) -> Self {
        let template = template_set.get(&template).unwrap();
        Self {
            field: Minefield::new_shaped(
                |&position| {
                    commands
                        .spawn(ServerTileBundle {
                            tile: ServerTile::Empty,
                            position,
                            owner: Owner(owner),
                        })
                        .id()
                },
                template,
            ),
            template: template.clone(),
            selections: default(),
            state: AreaAttack::Selecting,
            typed_marker: AreaAttackServer,
            stage_timer: default(),
        }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub color: PlayerColor,
    pub position: Position,
    pub frozen: Frozen,
    pub killed: Killed,
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct Frozen(Option<Duration>);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct Killed(bool);

#[derive(Component, Debug, Clone, Copy)]
pub enum ServerTile {
    /// No one has claimed the tile, and the tile does not contain a mine
    Empty,
    /// A player has claimed the tile
    Owned { player: Entity },
    /// There is a mine on this tile, and it has not been revealed
    Mine,
    /// There is a mine on this tile, and it has been revealed
    HardMine,
    /// The tile has been destroyed by being revealed as a mine in attack mode. It can no longer be
    /// recovered
    Destroyed,
}

#[derive(Component, Deref, Clone, Copy)]
pub struct Owner(Entity);

#[derive(Bundle)]
pub struct ServerTileBundle {
    pub tile: ServerTile,
    pub position: Position,
    pub owner: Owner,
}

#[derive(Component, Serialize, Deserialize, Clone, Copy)]
pub enum ClientTile {
    /// No one has claimed this tile, and it isn't known whether it is blank or contains a mine.
    Unknown,
    /// This tile has been claimed by the player specified by the given ID. In addition, if the
    /// client using this type is the one that owns this tile, it will know the number of cells
    /// neighboring this tile which have mines (`num_neighbors`), which can also be zero, in which
    /// case a filled tile without numbers will be shown. If this tile is not owned by the client,
    /// then this field will always be zero.
    Owned {
        player: Entity,
        num_neighbors: u8,
    },
    /// There is a mine on this tile which someone has revealed. Therefore it is no longer able to
    /// be claimed.
    Mine,
    Flag,
    /// The tile has been destroyed by being revealed as a mine in attack mode. It can no longer be
    /// recovered
    Destroyed,
}

#[derive(Bundle)]
pub struct ClientTileBundle {
    pub tile: ClientTile,
    pub position: Position,
    pub sprite: SpriteSheetBundle,
}

#[derive(Serialize, Deserialize, Clone, Copy, Component, EnumIter, PartialEq, Eq, Debug)]
pub enum PlayerColor {
    Yellow,
    Green,
    Blue,
    Purple,
}

impl From<PlayerColor> for Color {
    fn from(id: PlayerColor) -> Self {
        match id {
            PlayerColor::Yellow => Color::YELLOW,
            PlayerColor::Green => Color::GREEN,
            PlayerColor::Blue => Color::BLUE,
            PlayerColor::Purple => Color::PURPLE,
        }
    }
}

#[derive(Component, Debug, Deref, DerefMut, Default)]
pub struct InitialSelections(pub HashMap<Entity, Position>);

#[derive(Debug, Clone, Copy)]
pub struct RevealTile {
    pub position: Position,
    pub player: Entity,
    pub game: Entity,
}

#[derive(Resource, Deref, DerefMut)]
pub struct FreezeTimer(Timer);

#[derive(Component)]
pub struct FreezeTimerDisplay;

impl Default for FreezeTimer {
    fn default() -> Self {
        Self(
            Timer::new(FREEZE_DURATION, TimerMode::Once)
                .tap_mut(|timer| timer.set_elapsed(FREEZE_DURATION)),
        )
    }
}
