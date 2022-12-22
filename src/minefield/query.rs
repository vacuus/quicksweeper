#![allow(non_snake_case)]

use std::ops::Deref;

use bevy::{prelude::*, ecs::{system::SystemParam, query::WorldQuery}};
use ouroboros::self_referencing;
use rand::Rng;

use crate::common::{Contains, Position};

use super::Minefield;

#[derive(SystemParam)]
pub struct MinefieldQuery<'w, 's, T>
where
    T: WorldQuery + 'static,
{
    minefield_query: Query<'w, 's, &'static Minefield>,
    tile_query: Query<'w, 's, T>,
}

impl<'a, 'w, 's, T> MinefieldQuery<'w, 's, T>
where
    T: WorldQuery + 'static,
    'a: 'w + 's,
{
    fn get(self, entity: Entity) -> Option<AdjoinedMinefield<'a, 'w, 's, T>> {
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
struct BorrowedMinefield<'a, 'w, 's>
where
    'w: 'a,
    's: 'a,
{
    minefield_query: Query<'w, 's, &'static Minefield>,
    #[borrows(minefield_query)]
    minefield: &'this Minefield,
    _phantom: std::marker::PhantomData< &'a()>,
}

impl<'a, 'w, 's> Deref for BorrowedMinefield<'a, 'w, 's> {
    type Target = Minefield;

    fn deref(&self) -> &Self::Target {
        self.borrow_minefield()
    }
}

struct AdjoinedMinefield<'a, 'w, 's, T>
where
    T: WorldQuery,
    'w: 'a,
    's: 'a,
{
    minefield: BorrowedMinefield<'a, 'w, 's>,
    tile_query: Query<'w, 's, T>,
}

impl<'a, 'w, 's, T> AdjoinedMinefield<'a, 'w, 's, T>
where
    T: WorldQuery,
    'w: 'a,
    's: 'a,
{
    pub fn choose_multiple(
        &'a mut self,
        exclude: &'a impl Contains<Position>,
        rng: &'a mut impl Rng,
        mut op: impl for<'u> FnMut(Position, <T as WorldQuery>::Item<'u>),
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

    pub fn get_tile(&'a mut self, position: &Position) -> <T as WorldQuery>::Item<'a> {
        self.tile_query.get_mut(self.minefield[position]).unwrap()
    }
}
