#![allow(non_snake_case)]

use std::ops::Deref;

use bevy::{
    ecs::{
        query::{QueryItem, ROQueryItem, WorldQuery},
        system::SystemParam,
    },
    prelude::*,
};
use gridly::prelude::*;
use ouroboros::self_referencing;
use rand::Rng;

use crate::common::{Contains, Position};

use super::Minefield;

#[derive(SystemParam)]
pub struct MinefieldQuery<'w, 's, Tile>
where
    Tile: WorldQuery + 'static,
{
    minefield_query: Query<'w, 's, &'static Minefield>,
    tile_query: Query<'w, 's, Tile>,
}

impl<'all, 'world, 'state, Tile> MinefieldQuery<'world, 'state, Tile>
where
    Tile: WorldQuery + 'static,
    'world: 'all,
    'state: 'all,
{
    pub fn get(
        &'all mut self,
        entity: Entity,
    ) -> Option<AdjoinedMinefield<'all, 'world, 'state, Tile>> {
        self.minefield_query
            .contains(entity)
            .then_some(AdjoinedMinefield {
                tile_query: &mut self.tile_query,
                minefield: BorrowedMinefieldBuilder {
                    minefield_query: &self.minefield_query,
                    minefield_builder: |query: &&Query<&Minefield>| query.get(entity).unwrap(),
                }
                .build(),
            })
    }

    pub fn get_single(&'all mut self) -> Option<AdjoinedMinefield<'all, 'world, 'state, Tile>> {
        self.minefield_query
            .get_single()
            .is_ok()
            .then(|| AdjoinedMinefield {
                tile_query: &mut self.tile_query,
                minefield: BorrowedMinefieldBuilder {
                    minefield_query: &mut self.minefield_query,
                    minefield_builder: |query: &&Query<&Minefield>| query.single(),
                }
                .build(),
            })
    }
}

#[self_referencing]
struct BorrowedMinefield<'all, 'world, 'state>
where
    'world: 'all,
    'state: 'all,
{
    minefield_query: &'all Query<'world, 'state, &'static Minefield>,
    #[borrows(minefield_query)]
    minefield: &'this Minefield,
}

impl<'all, 'world, 'state> Deref for BorrowedMinefield<'all, 'world, 'state> {
    type Target = Minefield;

    fn deref(&self) -> &Self::Target {
        self.borrow_minefield()
    }
}

pub struct AdjoinedMinefield<'all, 'world, 'state, Tile>
where
    Tile: WorldQuery,
    'world: 'all,
    'state: 'all,
{
    minefield: BorrowedMinefield<'all, 'world, 'state>,
    tile_query: &'all mut Query<'world, 'state, Tile>,
}

impl<'all, 'world, 'state, 'query, Tile> AdjoinedMinefield<'all, 'world, 'state, Tile>
where
    Tile: WorldQuery,
    'world: 'all,
    'state: 'all,
    'all: 'query,
{
    pub fn choose_multiple(
        &'query mut self,
        exclude: &'all impl Contains<Position>,
        rng: &'all mut impl Rng,
        mut op: impl for<'every> FnMut(Position, QueryItem<'every, Tile>),
    ) {
        self.minefield
            .choose_multiple(exclude, rng)
            .into_iter()
            .map(|(loc, u)| (Position::from(*loc), u))
            .for_each(|(pos, tile_id)| {
                let tile = self.tile_query.get_mut(tile_id).unwrap();
                op(pos, tile)
            });
    }

    pub fn iter_neighbors_enumerated(
        &self,
        position: Position,
    ) -> impl Iterator<Item = (Position, ROQueryItem<Tile>)> {
        self.minefield
            .iter_neighbors_enumerated(position)
            .map(|(loc, tile_id)| {
                let world_access = self.tile_query.get(tile_id).unwrap();
                (loc, world_access)
            })
    }

    pub fn iter_neighbor_positions(
        &self,
        position: Position,
    ) -> impl Iterator<Item = Position> + '_ {
        self.minefield.iter_neighbor_positions(position)
    }

    pub fn iter_neighbors(&self, position: Position) -> impl Iterator<Item = ROQueryItem<Tile>> {
        self.iter_neighbors_enumerated(position)
            .map(|(_, tile)| tile)
    }

    pub fn get(&self, position: Position) -> Option<ROQueryItem<Tile>> {
        self.minefield
            .get(position)
            .map(|tile_id| self.tile_query.get(tile_id.unwrap()).unwrap())
            .ok()
    }

    pub fn get_mut(&'query mut self, position: Position) -> Option<QueryItem<'query, Tile>> {
        // self.tile_query.get_mut(self.minefield[&position]).unwrap()
        self.minefield
            .get(position)
            .map(|tile_id| self.tile_query.get_mut(tile_id.unwrap()).unwrap())
            .ok()
    }
}
