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

    /// Marches the edge of the circle with the given radius centered at the origin. For a radius AB
    /// with the circle center at A, the iterator starts at B. From there, for each integral point C
    /// on the radius, the iterator yields C paired with the length of the line segment along the
    /// radius' perpendicular from C to the edge of the circle.
    fn march_circle_edge(radius: isize) -> impl Iterator<Item = (isize, isize)> {
        let contained_square_radius = (radius as f32 / 2f32.sqrt()).round() as isize;
        (0..=radius)
            .rev()
            // find the furthest extension possible for each grade
            .scan(0, move |extension, grade| {
                *extension = (*extension..)
                    // extensions should not go past the perimeter of the circle
                    .take_while(|extension| extension * extension + grade * grade < radius * radius)
                    .last()
                    .unwrap_or(*extension);

                // // disallow the contained square to portrude from the circle
                if *extension == contained_square_radius && grade == contained_square_radius {
                    *extension -= 1;
                }

                Some((grade, *extension))
            })
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
        let segments = Self::march_circle_edge(radius)
            // the radius of the circle is not contained, but the edge of the contained square is
            .skip(1)
            .take_while(move |(r, _)| *r >= contained_square_radius)
            // fill in the extensions from each grade
            .flat_map(|(grade, max_extension)| {
                (1..=max_extension).map(move |extension| (grade, extension))
            })
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

        let contained_squares = (1..contained_square_radius)
            .cartesian_product(1..contained_square_radius)
            .flat_map(|(x, y)| [(x, y), (-x, y), (x, -y), (-x, -y)]);

        let s = *self;
        segments
            .chain(radii)
            .chain(contained_squares)
            .map(move |(x, y)| s + Position::new(x, y))
    }

    /// Iterates over all of the neighbors of the disk -- that is, all the tiles which are at most
    /// one away from the disk without being on the disk
    pub fn disk_neighbors(&self, radius: usize) -> impl Iterator<Item = Position> {
        let s = *self;
        let radius = radius as isize;
        Self::march_circle_edge(radius)
            .tuple_windows::<(_, _)>()
            .flat_map(|((grade, start_ex), (_, end_ex))| {
                (start_ex + 1..=end_ex + 1).map(move |x| (grade, x))
            })
            .flat_map(|(x, y)| [(x, y), (-x, y), (x, -y), (-x, -y)])
            // add the caps on the top of the circle
            .chain(
                [(radius + 1, 1), (radius + 1, 0), (radius + 1, -1)]
                    .into_iter()
                    .flat_map(|(x, y)| [(x, y), (-x, -y), (y, x), (-y, -x)]),
            )
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
    commands.spawn(Camera3dBundle {
        transform: Transform {
            translation: Vec3::new(0., 75., 0.),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                .mul_quat(Quat::from_rotation_z(std::f32::consts::PI)),
            ..default()
        },
        ..default()
    });
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

pub trait Vec2Ext {
    /// Extends this vector in the y direction, treating the original vector as if it were flat on
    /// the xz plane. This is useful for manipulating vectors which act on grids flat on the xz
    /// plane.
    fn extend_xz(&self, y: f32) -> Vec3;
}

impl Vec2Ext for Vec2 {
    fn extend_xz(&self, y: f32) -> Vec3 {
        Vec3::new(self.x, y, self.y)
    }
}

#[derive(Component, Deref)]
pub struct NeedsMaterial(pub Handle<StandardMaterial>);

pub fn insert_materials(
    mut commands: Commands,
    mut new_meshes: Query<(Entity, &mut Handle<StandardMaterial>, &Name), Added<Handle<Mesh>>>,
    color_requested: Query<&NeedsMaterial>,
    parents: Query<&Parent>,
) {
    let parent_of = |mut id: Entity, recursions: usize| {
        for _ in 0..recursions {
            id = **parents.get(id).unwrap()
        }
        id
    };

    for (tile_id, mut material, name) in &mut new_meshes {
        if name.contains("Text") || &**name == ("tile_empty") {
            continue;
        }
        let root = parent_of(tile_id, 3);
        if let Ok(color) = color_requested.get(root) {
            *material = (*color).clone();
        }
        commands.entity(root).remove::<NeedsMaterial>(); // TODO: Make this assignable to multiple meshes within a scene
    }
}

pub struct QuicksweeperTypes;

impl Plugin for QuicksweeperTypes {
    fn build(&self, app: &mut App) {
        app.init_resource::<Events<CheckCell>>()
            .insert_resource(ClearColor(Color::BLACK))
            .add_event::<FlagCell>()
            .add_event::<InitCheckCell>()
            .add_startup_system(init_cameras)
            .add_system(insert_materials);
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
            "disk 8: {:?}",
            Position::ZERO
                .radius(8)
                .map(|Position { x, y }| (x, y))
                .collect_vec()
        );
        println!(
            "perimeter 8: {:?}",
            Position::ZERO
                .disk_neighbors(8)
                .map(|Position { x, y }| (x, y))
                .collect_vec()
        );
        println!(
            "disk 4: {:?}",
            Position::ZERO
                .radius(4)
                .map(|Position { x, y }| (x, y))
                .collect_vec()
        );
        println!(
            "disk 10: {:?}",
            Position::ZERO
                .radius(10)
                .map(|Position { x, y }| (x, y))
                .collect_vec()
        );

        let mut unique = HashSet::new();
        let mut duplicates = Vec::new(); // unfortunately this vec itself isn't necessarily deduplicated
        for p @ Position { x, y } in Position::ZERO.radius(10) {
            if !unique.insert(p) {
                duplicates.push((x, y));
            }
        }

        assert_eq!(
            duplicates.len(),
            0,
            "duplicated (may contain duplicates itself): {duplicates:?}",
        )
    }
}
