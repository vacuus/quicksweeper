use bevy::prelude::*;

use crate::{common::Position, cursor::Cursor, load::Textures};
use tap::Tap;

/// Size of a single cell containing or not containing a mine. For now the display size of the mine
/// will be kept the same as the actual size of the sprite, but of course this will be subject to
/// change.
pub const TILE_SIZE: f32 = 32.0;

#[derive(Bundle)]
pub struct MineCell {
    sprite: SceneBundle,
    state: MineCellState,
    position: Position,
}

impl MineCell {
    pub fn new_empty(position @ Position { x, y }: Position, textures: &Res<Textures>) -> Self {
        MineCell {
            sprite: SceneBundle {
                scene: textures.tile_empty.clone(),
                transform: Transform::from_translation(Vec3::new(
                    (x as f32) * TILE_SIZE,
                    0.0,
                    (y as f32) * TILE_SIZE,
                )),
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

pub fn render_mines(
    // do it!
    mut changed_cells: Query<
        (&mut TextureAtlasSprite, &MineCellState),
        Or<(Added<MineCellState>, Changed<MineCellState>)>,
    >,
    cursor: Query<&Cursor>,
) {
    let color = cursor.get_single().map(|c| c.color).unwrap_or(Color::WHITE);
    changed_cells.for_each_mut(|(mut sprite, state)| {
        *sprite = match state {
            MineCellState::Empty | MineCellState::Mine => TextureAtlasSprite::new(9),
            MineCellState::FlaggedMine | MineCellState::FlaggedEmpty => {
                TextureAtlasSprite::new(10).tap_mut(|s| s.color = color)
            }
            &MineCellState::Revealed(x) => {
                TextureAtlasSprite::new(x as usize).tap_mut(|s| s.color = color)
            }
        };
    })
}
