#![allow(non_snake_case)]

use std::ops::Deref;

use bevy::{
    ecs::{query::WorldQuery, system::SystemParam},
    prelude::*,
};
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
    'all: 'world + 'state,
{
    fn get(self, entity: Entity) -> Option<AdjoinedMinefield<'all, 'world, 'state, Tile>> {
        self.minefield_query
            .contains(entity)
            .then_some(AdjoinedMinefield {
                tile_query: self.tile_query,
                minefield: BorrowedMinefieldBuilder {
                    minefield_query: self.minefield_query,
                    minefield_builder: |query: &Query<&Minefield>| query.get(entity).unwrap(),
                    _phantom: default(),
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
    minefield_query: Query<'world, 'state, &'static Minefield>,
    #[borrows(minefield_query)]
    minefield: &'this Minefield,
    _phantom: std::marker::PhantomData<&'all ()>,
}

impl<'all, 'world, 'state> Deref for BorrowedMinefield<'all, 'world, 'state> {
    type Target = Minefield;

    fn deref(&self) -> &Self::Target {
        self.borrow_minefield()
    }
}

struct AdjoinedMinefield<'all, 'world, 'state, Tile>
where
    Tile: WorldQuery,
    'world: 'all,
    'state: 'all,
{
    minefield: BorrowedMinefield<'all, 'world, 'state>,
    tile_query: Query<'world, 'state, Tile>,
}

impl<'all, 'world, 'state, Tile> AdjoinedMinefield<'all, 'world, 'state, Tile>
where
    Tile: WorldQuery,
    'world: 'all,
    'state: 'all,
{
    pub fn choose_multiple(
        &'all mut self,
        exclude: &'all impl Contains<Position>,
        rng: &'all mut impl Rng,
        mut op: impl for<'every> FnMut(Position, <Tile as WorldQuery>::Item<'every>),
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

    pub fn get_tile(&'all mut self, position: &Position) -> <Tile as WorldQuery>::Item<'all> {
        self.tile_query.get_mut(self.minefield[position]).unwrap()
    }
}
