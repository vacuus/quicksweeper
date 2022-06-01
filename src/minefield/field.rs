use std::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

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
    Flagged,
}

#[derive(Component)]
struct Mine(Position);

#[derive(Deref, DerefMut, Component)]
pub struct Minefield(pub(super) Array2D<MineCell>);

struct MineNeighbors<'a> {
    backing: &'a Minefield,
    items: [Position; 8],
    size: u8,
}

impl<'a> Iterator for MineNeighbors<'a> {
    type Item = (Position, &'a MineCell);

    fn next(&mut self) -> Option<Self::Item> {
        (self.size != 0).then(|| {
            self.size -= 1;
            let pos = std::mem::replace(&mut self.items[self.size as usize], unsafe {
                MaybeUninit::uninit().assume_init()
            });
            (pos.clone(), &self.backing[pos])
        })
    }
}

impl Minefield {
    pub fn iter_neighbors_enumerated(
        &self,
        pos: Position,
    ) -> impl Iterator<Item = (Position, &MineCell)> + '_ {
        let mut size = 0;
        let mut items: [Position; 8] = unsafe { MaybeUninit::uninit().assume_init() };

        let max_x = (self.num_columns() - 1) as u32;
        let max_y = (self.num_rows() - 1) as u32;

        // left
        if pos.0 != 0 {
            // lower left
            if pos.1 != 0 {
                items[size] = Position(pos.0 - 1, pos.1 - 1);
                size += 1;
            }
            // upper left
            if pos.1 != max_y {
                items[size] = Position(pos.0 - 1, pos.1 + 1);
                size += 1;
            }
            items[size] = Position(pos.0 - 1, pos.1);
            size += 1;
        }
        // right
        if pos.0 != max_x {
            // lower right
            if pos.1 != 0 {
                items[size] = Position(pos.0 + 1, pos.1 - 1);
                size += 1;
            }
            // upper right
            if pos.1 != max_y {
                items[size] = Position(pos.0 + 1, pos.1 + 1);
                size += 1;
            }
            items[size] = Position(pos.0 + 1, pos.1);
            size += 1;
        }
        // bottom
        if pos.1 != 0 {
            items[size] = Position(pos.0, pos.1 - 1);
            size += 1;
        }
        // top
        if pos.1 != max_y {
            items[size] = Position(pos.0, pos.1 + 1);
            size += 1;
        }

        MineNeighbors {
            backing: &self,
            items,
            size: size as u8,
        }
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
