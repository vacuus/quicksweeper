use bevy::{gltf::Gltf, prelude::*};

use crate::{
    common::{Position, Vec2Ext},
    cursor::Cursor,
    load::Textures,
};
use tap::Tap;

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

pub fn render_mines(
    // do it!
    mut changed_cells: Query<
        (&mut Handle<Scene>, &MineCellState),
        Or<(Added<MineCellState>, Changed<MineCellState>)>,
    >,
    cursor: Query<&Cursor>,
    textures: Res<Textures>,
    gltf: Res<Assets<Gltf>>,
) {
    let color = cursor.get_single().map(|c| c.color).unwrap_or(Color::WHITE);
    changed_cells.for_each_mut(|(mut scene, state)| {
        *scene = match state {
            MineCellState::Empty | MineCellState::Mine => textures.tile_empty.clone(),
            MineCellState::FlaggedMine | MineCellState::FlaggedEmpty => {
                // TextureAtlasSprite::new(10).tap_mut(|s| s.color = color)
                textures.tile_empty.clone()
            }
            &MineCellState::Revealed(x) => {
                // TextureAtlasSprite::new(x as usize).tap_mut(|s| s.color = color)
                gltf.get(&textures.mines_3d).unwrap().named_scenes[&format!("tile_filled.{x}")]
                    .clone()
            }
        };
    })
}
