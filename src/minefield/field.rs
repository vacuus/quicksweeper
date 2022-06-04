use std::ops::{Index, IndexMut};

use array2d::Array2D;
use bevy::{math::XY, prelude::*};
use tap::Tap;

use crate::{common::Position, textures::MineTextures};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug)]
pub struct MineCell {
    sprite: Entity,
    state: MineCellState,
    modified: bool,
}

impl MineCell {
    pub fn state(&self) -> &MineCellState {
        &self.state
    }

    pub fn set_state(&mut self, state: MineCellState) {
        self.modified = true;
        self.state = state;
    }

    pub fn is_flagged(&self) -> bool {
        match self.state {
            MineCellState::FlaggedEmpty | MineCellState::FlaggedMine => true,
            _ => false,
        }
    }

    pub fn new_empty(
        commands: &mut Commands,
        Position(XY { x, y }): Position,
        textures: &Res<MineTextures>,
    ) -> Self {
        let sprite = commands
            .spawn_bundle(textures.empty().tap_mut(|b| {
                b.transform = Transform {
                    translation: Vec3::new(x as f32 * 32.0, y as f32 * 32.0, 3.0),
                    ..Default::default()
                };
            }))
            .id();

        MineCell {
            sprite,
            state: MineCellState::Empty,
            modified: false,
        }
    }
}

pub fn render_mines(mut q: Query<&mut Minefield>, mut sprites: Query<&mut TextureAtlasSprite>) {
    q.single_mut().for_each_mut(|cell| {
        if cell.modified == true {
            cell.modified = false;
            *sprites.get_mut(cell.sprite).unwrap() = match cell.state {
                MineCellState::Empty | MineCellState::Mine => TextureAtlasSprite::new(9),
                MineCellState::FlaggedMine | MineCellState::FlaggedEmpty => {
                    TextureAtlasSprite::new(10)
                }
                MineCellState::FoundEmpty(x) => TextureAtlasSprite::new(x as usize),
            }
        }
    });
}

pub fn display_mines(mut q: Query<&mut Minefield>, mut sprites: Query<&mut TextureAtlasSprite>) {
    q.single_mut().for_each_mut(|cell| {
        if cell.state == MineCellState::Mine {
            *sprites.get_mut(cell.sprite).unwrap() = TextureAtlasSprite::new(11)
        }
    });
}

#[derive(Clone, Debug, PartialEq)]
pub enum MineCellState {
    Empty,
    Mine,
    FoundEmpty(u8),
    FlaggedEmpty,
    FlaggedMine,
}

#[derive(Component)]
struct Mine(Position);

#[derive(Component)]
pub struct Minefield {
    pub(super) field: Array2D<MineCell>,
    pub(super) remaining_blank: usize,
}

impl Deref for Minefield {
    type Target = Array2D<MineCell>;

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
    ) -> impl Iterator<Item = (Position, &MineCell)> + '_ {
        self.iter_neighbor_positions(pos)
            .map(|pos| (pos.clone(), &self[pos]))
    }

    pub fn iter_neighbor_positions(&self, pos: Position) -> impl Iterator<Item = Position> {
        pos.iter_neighbors(self.num_columns() as u32, self.num_rows() as u32)
    }

    pub fn for_each_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut MineCell),
    {
        let cols = self.num_columns();
        for ix in (0..self.num_elements()).map(|pos| (pos / cols, pos % cols)) {
            f(&mut (**self)[ix])
        }
    }
}

impl Index<Position> for Minefield {
    type Output = MineCell;

    fn index(&self, Position(XY { x: y, y: x }): Position) -> &Self::Output {
        &(**self)[(x as usize, y as usize)]
    }
}

impl IndexMut<Position> for Minefield {
    fn index_mut(&mut self, Position(XY { x: y, y: x }): Position) -> &mut Self::Output {
        &mut (**self)[(x as usize, y as usize)]
    }
}
