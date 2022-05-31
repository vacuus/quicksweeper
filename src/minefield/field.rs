use std::ops::{Index, IndexMut};

use array2d::Array2D;
use bevy::prelude::*;

use crate::common::Position;

#[derive(Clone, Debug)]
pub struct MineCell {
    pub sprite: Entity,
    pub state: MineCellState,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MineCellState {
    Empty,
    Mine,
    MarkedEmpty(u8),
    MarkedMine,
}

#[derive(Component)]
struct Mine(Position);

#[derive(Deref, DerefMut, Component)]
pub struct Minefield(pub(super) Array2D<MineCell>);

impl Index<Position> for Minefield {
    type Output = MineCell;

    fn index(&self, Position(x, y): Position) -> &Self::Output {
        &(**self)[(x, y)]
    }
}

impl IndexMut<Position> for Minefield {
    fn index_mut(&mut self, Position(x, y): Position) -> &mut Self::Output {
        &mut (**self)[(x, y)]
    }
}