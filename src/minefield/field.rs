use std::ops::Index;

use bevy::prelude::*;
use gridly::{
    prelude::*,
    vector::{Columns, Rows},
};
use gridly_grids::SparseGrid;
use rand::{seq::IteratorRandom, Rng};

use crate::common::{Contains, Position};
use std::ops::{Deref, DerefMut};

use super::FieldShape;

#[derive(Component)]
pub struct Minefield {
    pub field: SparseGrid<Option<Entity>>,
    pub remaining_blank: usize,
}

impl Deref for Minefield {
    type Target = SparseGrid<Option<Entity>>;

    fn deref(&self) -> &Self::Target {
        &self.field
    }
}

impl DerefMut for Minefield {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.field
    }
}

/// Converts from the total amount of cells there are on the board to the amount of cells without
/// mines there should be on the board
fn field_density(val: usize) -> usize {
    val * 8 / 10
}

impl Minefield {
    pub fn new_shaped<F>(mut make_entity: F, template: &FieldShape) -> Self
    where
        F: FnMut(&Position) -> Entity,
    {
        let mut field = SparseGrid::new_default((Rows(10), Columns(10)), None); // TODO: Use less arbitrary numbers in init
        for pos in template.decode() {
            let entity = make_entity(&pos);
            field.insert(pos, Some(entity));
        }

        let amnt_cells = field.occupied_entries().count();

        Self {
            remaining_blank: field_density(amnt_cells),
            field,
        }
    }

    pub fn choose_multiple(
        &self,
        exclude: &impl Contains<Position>,
        rng: &mut impl Rng,
    ) -> impl IntoIterator<Item = (&Location, Entity)> {
        let num_entries = self.field.occupied_entries().count();

        self.occupied_entries()
            .filter_map(|(a, b)| b.map(|b| (a, b)))
            .filter(|&(&pos, _)| !exclude.contains(&pos.into()))
            .choose_multiple(rng, num_entries - self.remaining_blank)
    }

    pub fn iter_positions(&self) -> impl Iterator<Item = Position> + '_ {
        self.occupied_entries().map(|(&loc, _)| loc.into())
    }

    pub fn iter_neighbors_enumerated(
        &self,
        pos: Position,
    ) -> impl Iterator<Item = (Position, Entity)> + '_ {
        pos.neighbors().into_iter().filter_map(|neighbor| {
            self.get(neighbor)
                .ok()
                .and_then(|&x| x)
                .map(|entity| (neighbor, entity))
        })
    }

    pub fn iter_neighbor_positions(&self, pos: Position) -> impl Iterator<Item = Position> + '_ {
        self.iter_neighbors_enumerated(pos).map(|(pos, _)| pos)
    }

    pub fn is_contained(&self, pos: &Position) -> bool {
        matches!(self.get(pos), Ok(Some(_)))
    }
}

impl Index<&Position> for Minefield {
    type Output = Entity;

    fn index(&self, pos: &Position) -> &Self::Output {
        self.field[pos].as_ref().unwrap()
    }
}
