use std::{collections::HashMap, ops::Index};

use bevy::{math::XY, prelude::*};
use tap::Tap;

use crate::{common::Position, load::MineTextures};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Bundle)]
pub struct MineCell {
    #[bundle]
    sprite: SpriteSheetBundle,
    state: MineCellState,
    position: Position,
}

impl MineCell {
    pub fn new_empty(
        position @ Position(XY { x, y }): Position,
        textures: &Res<MineTextures>,
    ) -> Self {
        MineCell {
            sprite: textures.empty().tap_mut(|b| {
                b.transform = Transform {
                    translation: Vec3::new(x as f32 * 32.0, y as f32 * 32.0, 3.0),
                    ..Default::default()
                };
            }),
            state: MineCellState::Empty,
            position,
        }
    }
}

pub fn render_mines(
    mut changed_cells: Query<
        (&mut TextureAtlasSprite, &MineCellState),
        Or<(Added<MineCellState>, Changed<MineCellState>)>,
    >,
) {
    changed_cells.for_each_mut(|(mut sprite, state)| {
        *sprite = match state {
            MineCellState::Empty | MineCellState::Mine => TextureAtlasSprite::new(9),
            MineCellState::FlaggedMine | MineCellState::FlaggedEmpty => TextureAtlasSprite::new(10),
            MineCellState::FoundEmpty(x) => TextureAtlasSprite::new(*x as usize),
        };
    })
}

pub fn display_mines(mut cells: Query<(&mut TextureAtlasSprite, &MineCellState)>) {
    cells.for_each_mut(|(mut sprite, state)| {
        if *state == MineCellState::Mine {
            *sprite = TextureAtlasSprite::new(11)
        }
    });
}

#[derive(Clone, Debug, PartialEq, Component)]
pub enum MineCellState {
    Empty,
    Mine,
    FoundEmpty(u8),
    FlaggedEmpty,
    FlaggedMine,
}

impl MineCellState {
    pub fn is_flagged(&self) -> bool {
        match self {
            MineCellState::FlaggedEmpty | MineCellState::FlaggedMine => true,
            _ => false,
        }
    }

    pub fn is_mine(&self) -> bool {
        match self {
            MineCellState::Mine | MineCellState::FlaggedMine => true,
            _ => false,
        }
    }

    pub fn is_marked(&self) -> bool {
        match self {
            MineCellState::Mine | MineCellState::Empty => false,
            _ => true,
        }
    }
}

#[derive(Component)]
pub struct Minefield {
    pub(super) field: HashMap<Position, Entity>,
    pub(super) remaining_blank: usize,
}

impl Deref for Minefield {
    type Target = HashMap<Position, Entity>;

    fn deref(&self) -> &Self::Target {
        &self.field
    }
}

impl DerefMut for Minefield {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.field
    }
}

impl Minefield {
    pub fn iter_neighbors_enumerated(
        &self,
        pos: Position,
    ) -> impl Iterator<Item = (Position, Entity)> + '_ {
        pos.iter_neighbors().filter_map(move |neighbor| {
            self.get(&neighbor).map(|entity| (neighbor, entity.clone()))
        })
    }

    pub fn iter_neighbor_positions(&self, pos: Position) -> impl Iterator<Item = Position> + '_ {
        self.iter_neighbors_enumerated(pos).map(|(pos, _)| pos)
    }
}

impl Index<Position> for Minefield {
    type Output = Entity;

    fn index(&self, pos: Position) -> &Self::Output {
        &(**self)[&pos]
    }
}
