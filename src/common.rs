use bevy::{math::XY, prelude::*};
use derive_more::{Deref, DerefMut};
use iyes_loopless::prelude::*;
use rand::{prelude::StdRng, SeedableRng};
use std::mem::MaybeUninit;

use crate::AppState;

#[derive(PartialEq, Clone, Debug, Deref, DerefMut)]
pub struct Position(pub XY<u32>);

pub struct PositionNeighborsIter {
    items: [Position; 8],
    size: u8,
}

impl Iterator for PositionNeighborsIter {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        (self.size != 0).then(|| {
            self.size -= 1;
            self.items[self.size as usize].clone()
        })
    }
}

impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Self(XY { x, y })
    }

    pub fn iter_neighbors(&self, max_x: u32, max_y: u32) -> PositionNeighborsIter {
        let (max_x, max_y) = (max_x - 1, max_y - 1);
        let mut size = 0;
        let mut items: [Position; 8] = unsafe { MaybeUninit::uninit().assume_init() };

        // left
        if self.x != 0 {
            // lower left
            if self.y != 0 {
                items[size] = Self::new(self.x - 1, self.y - 1);
                size += 1;
            }
            // upper left
            if self.y != max_y {
                items[size] = Self::new(self.x - 1, self.y + 1);
                size += 1;
            }
            items[size] = Self::new(self.x - 1, self.y);
            size += 1;
        }
        // right
        if self.x != max_x {
            // lower right
            if self.y != 0 {
                items[size] = Self::new(self.x + 1, self.y - 1);
                size += 1;
            }
            // upper right
            if self.y != max_y {
                items[size] = Self::new(self.x + 1, self.y + 1);
                size += 1;
            }
            items[size] = Self::new(self.x + 1, self.y);
            size += 1;
        }
        // bottom
        if self.y != 0 {
            items[size] = Self::new(self.x, self.y - 1);
            size += 1;
        }
        // top
        if self.y != max_y {
            items[size] = Self::new(self.x, self.y + 1);
            size += 1;
        }

        PositionNeighborsIter {
            items,
            size: size as u8,
        }
    }

    pub fn absolute(&self, size_x: f32, size_y: f32) -> Vec2 {
        Vec2::new(self.x as f32 * size_x, self.y as f32 * size_y)
    }
}

#[derive(Clone, Debug)]
pub struct CheckCell(pub Position);

#[derive(Clone, Debug)]
pub struct InitCheckCell(pub Position);

#[derive(Clone, Debug)]
pub struct FlagCell(pub Position);

pub struct GameInitData {
    pub rows: u32,
    pub cols: u32,
}

pub struct QuicksweeperTypes;

impl Plugin for QuicksweeperTypes {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(AppState::Loading)
            .insert_resource(StdRng::from_entropy())
            .add_event::<CheckCell>()
            .add_event::<FlagCell>()
            .add_event::<InitCheckCell>()
            // temporary addition
            .insert_resource(GameInitData {
                rows: 30,
                cols: 50,
            });
    }
}
