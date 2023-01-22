use bevy::prelude::*;

use crate::{
    common::{Position, Vec2Ext},
    load::Textures,
};

/// Size of a single cell containing or not containing a mine. For now the display size of the mine
/// will be kept the same as the actual size of the sprite, but of course this will be subject to
/// change.
pub const TILE_SIZE: f32 = 2.0;

#[derive(Bundle)]
pub struct MineCell {
    sprite: SceneBundle,
    state: MineCellState,
    position: Position,
}

impl MineCell {
    pub fn new_empty(position: Position, textures: &Res<Textures>) -> Self {
        MineCell {
            sprite: SceneBundle {
                scene: textures.tile_empty.clone(),
                transform: Transform::from_translation(
                    position.absolute(TILE_SIZE, TILE_SIZE).extend_xz(0.0),
                ),
                ..default()
            },
            state: MineCellState::Empty,
            position,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Component)]
pub enum MineCellState {
    Empty,
    Mine,
    Revealed(u8),
    FlaggedEmpty,
    FlaggedMine,
}

impl MineCellState {
    pub fn is_flagged(&self) -> bool {
        matches!(
            self,
            MineCellState::FlaggedEmpty | MineCellState::FlaggedMine
        )
    }

    pub fn is_mine(&self) -> bool {
        matches!(self, MineCellState::Mine | MineCellState::FlaggedMine)
    }

    pub fn is_marked(&self) -> bool {
        !matches!(self, MineCellState::Mine | MineCellState::Empty)
    }
}
