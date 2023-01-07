use arrayvec::ArrayVec;
use bevy::prelude::*;
use gridly::prelude::*;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use std::{
    hash::Hash,
    ops::{Add, Deref, Div, Sub},
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
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: &Position) -> f32 {
        Vec2::distance(self.into(), other.into())
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

    /// Returns an iterator over all positions at most a radius r away from this position. Only
    /// accepts radii greater than 0. Excludes the center of the circle. Positions are guaranteed to
    /// appear exactly once.
    ///
    /// Implementation adapted from <https://stackoverflow.com/questions/60549803/60551485#60551485>
    pub fn radius(&self, radius: usize) -> impl Iterator<Item = Position> {
        let radius = radius as isize;
        let contained_square_radius = (radius as f32 / 2f32.sqrt()).round() as isize;

        // grade the section between the contained square and the circle
        // the radius of the circle and the edge of the contained square are not included
        let segments = (contained_square_radius + 1..radius)
            // start iterating from the circle, heading toward the square
            .rev()
            // find the furthest extension possible for each grade
            .scan(1, move |extension, grade| {
                *extension = (*extension..)
                    // extensions should not go past the perimeter of the circle
                    .take_while(|extension| extension * extension + grade * grade < radius * radius)
                    .last()
                    .unwrap_or(*extension);
                Some((grade, *extension))
            })
            // fill in the extensions from each grade
            .flat_map(|(grade, extension)| (1..=extension).map(move |extension| (grade, extension)))
            // octuple the half-segment so that all segments are covered
            .flat_map(|(x, y)| {
                [
                    (x, y),
                    (x, -y),
                    (-x, y),
                    (-x, -y),
                    (y, x),
                    (y, -x),
                    (-y, x),
                    (-y, -x),
                ]
            });

        let radii = (1..=radius).flat_map(|r| [(0, r), (r, 0), (0, -r), (-r, 0)]);

        let contained_squares = (1..=contained_square_radius)
            .cartesian_product(1..=contained_square_radius)
            .flat_map(|(x, y)| [(x, y), (-x, y), (x, -y), (-x, -y)]);

        let s = *self;
        segments
            .chain(radii)
            .chain(contained_squares)
            .map(move |(x, y)| s + Position::new(x, y))
    }

    pub fn local_group(&self) -> ArrayVec<Position, 9> {
        Direction::iter()
            .filter_map(|direction| self.neighbor_direction(direction))
            .chain(std::iter::once(*self))
            .collect()
    }

    pub fn absolute(&self, size_x: f32, size_y: f32) -> Vec2 {
        Vec2::new(self.x as f32 * size_x, self.y as f32 * size_y)
    }
}

impl From<&Position> for Vec2 {
    fn from(val: &Position) -> Self {
        Vec2::new(val.x as f32, val.y as f32)
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

pub trait Contains<T> {
    fn contains(&self, _: &T) -> bool;
}

impl<T: PartialEq> Contains<T> for [T] {
    fn contains(&self, t: &T) -> bool {
        self.contains(t)
    }
}

impl<U: PartialEq, T> Contains<U> for T
where
    T: Deref<Target = [U]>,
{
    fn contains(&self, t: &U) -> bool {
        <[U]>::contains(self, t)
    }
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

#[cfg(test)]
mod test {
    use bevy::utils::HashSet;
    use itertools::Itertools;

    use super::Position;

    #[test]
    fn position_radii() {
        println!(
            "{:?}",
            Position::ZERO
                .radius(8)
                .map(|Position { x, y }| (x, y))
                .collect_vec()
        );
        println!(
            "{:?}",
            Position::ZERO
                .radius(4)
                .map(|Position { x, y }| (x, y))
                .collect_vec()
        );
        println!(
            "{:?}",
            Position::ZERO
                .radius(10)
                .map(|Position { x, y }| (x, y))
                .collect_vec()
        );

        assert_eq!(
            Position::ZERO.radius(10).collect_vec().len(),
            Position::ZERO.radius(10).collect::<HashSet<_>>().len()
        )
    }
}
