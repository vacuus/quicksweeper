use bevy::{math::XY, prelude::*};
use derive_more::{Deref, DerefMut};
use iyes_loopless::prelude::*;

use std::{
    hash::Hash,
    mem::MaybeUninit,
    ops::{Add, Div, Sub},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::SingleplayerState;

#[derive(PartialEq, Clone, Copy, Debug, Deref, DerefMut, Component)]
pub struct Position(pub XY<u32>);

impl Eq for Position {}

#[allow(clippy::derive_hash_xor_eq)]
// cannot fix this for now, since XY doesn't allow for Hash impls
impl Hash for Position {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl Add<Self> for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Position::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub<Self> for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Position::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Div<u32> for Position {
    type Output = Self;

    fn div(self, rhs: u32) -> Self::Output {
        Position::new(self.x / rhs, self.y / rhs)
    }
}

pub struct PositionNeighborsIter {
    items: [Position; 8],
    size: u8,
}

impl Iterator for PositionNeighborsIter {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        (self.size != 0).then(|| {
            self.size -= 1;
            self.items[self.size as usize]
        })
    }
}

impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Self(XY { x, y })
    }

    pub fn neighbor_direction(&self, direction: Direction) -> Option<Position> {
        match direction {
            Direction::North => self.y.checked_add(1).map(|y| Position::new(self.x, y)),
            Direction::NorthEast => self
                .x
                .checked_add(1)
                .zip(self.y.checked_add(1))
                .map(|(x, y)| Position::new(x, y)),
            Direction::East => self.x.checked_add(1).map(|x| Position::new(x, self.y)),
            Direction::SouthEast => self
                .x
                .checked_add(1)
                .zip(self.y.checked_sub(1))
                .map(|(x, y)| Position::new(x, y)),
            Direction::South => self.y.checked_sub(1).map(|y| Position::new(self.x, y)),
            Direction::SouthWest => self
                .x
                .checked_sub(1)
                .zip(self.y.checked_sub(1))
                .map(|(x, y)| Position::new(x, y)),
            Direction::West => self.x.checked_sub(1).map(|x| Position::new(x, self.y)),
            Direction::NorthWest => self
                .x
                .checked_sub(1)
                .zip(self.y.checked_add(1))
                .map(|(x, y)| Position::new(x, y)),
        }
    }

    pub fn iter_neighbors(&self) -> PositionNeighborsIter {
        let mut items: [Position; 8] = unsafe { MaybeUninit::zeroed().assume_init() };
        let mut size = 0;
        Direction::iter().for_each(|direction| {
            if let Some(pos) = self.neighbor_direction(direction) {
                items[size as usize] = pos;
                size += 1;
            }
        });
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
pub struct CheckCell(pub Position, pub Entity);

#[derive(Clone, Debug)]
pub struct InitCheckCell(pub Position, pub Entity);

#[derive(Clone, Debug)]
pub struct FlagCell(pub Position, pub Entity);

pub struct CurrentMinefield(pub Entity);

#[derive(EnumIter, Clone, Debug)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

pub struct QuicksweeperTypes;

impl Plugin for QuicksweeperTypes {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(SingleplayerState::Loading)
            // .insert_resource(StdRng::from_entropy())
            .add_event::<CheckCell>()
            .add_event::<FlagCell>()
            .add_event::<InitCheckCell>();
    }
}
