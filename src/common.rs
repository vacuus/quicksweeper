use arrayvec::ArrayVec;
use bevy::prelude::*;
use gridly::prelude::*;
use serde::{Serialize, Deserialize};

use std::{
    hash::Hash,
    ops::{Add, Div, Sub},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::cursor::CursorPosition;

#[derive(PartialEq, Clone, Copy, Debug, Component, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}

impl From<Location> for Position {
    fn from(loc: Location) -> Self {
        Position {
            x: loc.column.0,
            y: loc.row.0,
        }
    }
}

impl LocationLike for Position {
    fn row(&self) -> Row {
        Row(self.y)
    }

    fn column(&self) -> Column {
        Column(self.x)
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

impl Div<isize> for Position {
    type Output = Self;

    fn div(self, rhs: isize) -> Self::Output {
        Position::new(self.x / rhs, self.y / rhs)
    }
}

impl Position {
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
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

    pub fn neighbors(&self) -> ArrayVec<Position, 8> {
        Direction::iter()
            .filter_map(|direction| self.neighbor_direction(direction))
            .collect()
    }

    pub fn absolute(&self, size_x: f32, size_y: f32) -> Vec2 {
        Vec2::new(self.x as f32 * size_x, self.y as f32 * size_y)
    }
}

#[derive(Clone, Debug)]
pub struct CheckCell(pub CursorPosition);

#[derive(Clone, Debug)]
pub struct InitCheckCell {
    pub minefield: Entity,
    pub positions: Vec<Position>,
}

#[derive(Clone, Debug)]
pub struct FlagCell(pub CursorPosition);

#[rustfmt::skip]
#[derive(EnumIter, Clone, Debug)]
pub enum Direction {
             North,
    NorthWest,    NorthEast,
West,                      East,
    SouthWest,    SouthEast,
             South,
}

fn init_cameras(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub struct QuicksweeperTypes;

impl Plugin for QuicksweeperTypes {
    fn build(&self, app: &mut App) {
        app.init_resource::<Events<CheckCell>>()
            .add_event::<FlagCell>()
            .add_event::<InitCheckCell>()
            .add_startup_system(init_cameras);
    }
}
