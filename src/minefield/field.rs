use std::ops::Index;

use bevy::prelude::*;
use gridly::{
    prelude::*,
    vector::{Columns, Rows},
};
use gridly_grids::SparseGrid;
use tap::Tap;

use crate::{common::Position, load::MineTextures};
use std::ops::{Deref, DerefMut};

use super::BlankField;

#[derive(Clone, Bundle)]
pub struct MineCell {
    #[bundle]
    sprite: SpriteSheetBundle,
    state: MineCellState,
    position: Position,
}

impl MineCell {
    pub fn new_empty(position @ Position { x, y }: Position, textures: &Res<MineTextures>) -> Self {
        MineCell {
            sprite: textures.empty().tap_mut(|b| {
                b.transform = Transform {
                    translation: Vec3::new(x as f32 * 32.0, y as f32 * 32.0, 3.0),
                    ..Default::default()
                };
            }),
            state: MineCellState::Empty,
            position,
        }
    }
}

pub fn render_mines(
    mut changed_cells: Query<
        (&mut TextureAtlasSprite, &MineCellState),
        Or<(Added<MineCellState>, Changed<MineCellState>)>,
    >,
) {
    changed_cells.for_each_mut(|(mut sprite, state)| {
        *sprite = match state {
            MineCellState::Empty | MineCellState::Mine => TextureAtlasSprite::new(9),
            MineCellState::FlaggedMine | MineCellState::FlaggedEmpty => TextureAtlasSprite::new(10),
            &MineCellState::Revealed(x) => TextureAtlasSprite::new(x as usize),
        };
    })
}

#[derive(Clone, Debug, PartialEq, Component)]
pub enum MineCellState {
    Empty,
    Mine,
    Revealed(u8),
    FlaggedEmpty,
    FlaggedMine,
}

impl MineCellState {
    pub fn is_flagged(&self) -> bool {
        matches!(
            self,
            MineCellState::FlaggedEmpty | MineCellState::FlaggedMine
        )
    }

    pub fn is_mine(&self) -> bool {
        matches!(self, MineCellState::Mine | MineCellState::FlaggedMine)
    }

    pub fn is_marked(&self) -> bool {
        !matches!(self, MineCellState::Mine | MineCellState::Empty)
    }
}

#[derive(Component)]
pub struct Minefield {
    pub(super) field: SparseGrid<Option<Entity>>,
    pub(super) remaining_blank: usize,
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
    pub fn new_blank_shaped(
        commands: &mut Commands,
        textures: &Res<MineTextures>,
        template: &BlankField,
    ) -> Self {
        let mut field = SparseGrid::new_default((Rows(10), Columns(10)), None); // TODO: Use less arbitrary numbers in init
        for &pos in template.iter() {
            let entity = commands
                .spawn_bundle(MineCell::new_empty(pos, textures))
                .id();
            field.insert(pos, Some(entity));
        }

        let amnt_cells = field.occupied_entries().count();

        Self {
            remaining_blank: field_density(amnt_cells),
            field,
        }
    }

    pub fn refresh(&mut self, states: &mut Query<&mut MineCellState>) {
        states.for_each_mut(|mut state| *state = MineCellState::Empty);

        let amnt_cells = self.occupied_entries().count();
        self.remaining_blank = field_density(amnt_cells);
    }

    pub fn iter_neighbors_enumerated(
        &self,
        pos: Position,
    ) -> impl Iterator<Item = (Position, Entity)> + '_ {
        pos.neighbors().into_iter().filter_map(move |neighbor| {
            self.get(&neighbor)
                .ok()
                .and_then(|&x| x)
                .map(|entity| (neighbor, entity))
        })
    }

    pub fn iter_neighbor_positions(&self, pos: Position) -> impl Iterator<Item = Position> + '_ {
        self.iter_neighbors_enumerated(pos).map(|(pos, _)| pos)
    }

    pub fn remaining_blank(&self) -> usize {
        self.remaining_blank
    }
}

impl Index<&Position> for Minefield {
    type Output = Entity;

    fn index(&self, pos: &Position) -> &Self::Output {
        self.field[pos].as_ref().unwrap()
    }
}
