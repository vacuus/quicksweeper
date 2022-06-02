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
    FoundEmpty(u8),
    FlaggedEmpty,
    FlaggedMine,
}

#[derive(Component)]
struct Mine(Position);

#[derive(Deref, DerefMut, Component)]
pub struct Minefield(pub(super) Array2D<MineCell>);

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
}

impl Index<Position> for Minefield {
    type Output = MineCell;

    fn index(&self, Position(y, x): Position) -> &Self::Output {
        &(**self)[(x as usize, y as usize)]
    }
}

impl IndexMut<Position> for Minefield {
    fn index_mut(&mut self, Position(y, x): Position) -> &mut Self::Output {
        &mut (**self)[(x as usize, y as usize)]
    }
}
