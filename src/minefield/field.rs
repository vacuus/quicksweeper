use std::ops::{Index, IndexMut};

use array2d::Array2D;
use bevy::prelude::*;
use tap::Tap;

use crate::{common::Position, textures::MineTextures};

#[derive(Clone, Debug)]
pub struct MineCell {
    sprite: Entity,
    state: MineCellState,
    modified: bool,
}

impl MineCell {
    pub fn state(&self) -> &MineCellState {
        &self.state
    }

    pub fn set_state(&mut self, state: MineCellState) {
        self.modified = true;
        self.state = state;
    }

    pub fn new_empty(
        commands: &mut Commands,
        Position(x, y): Position,
        textures: &Res<MineTextures>,
    ) -> Self {
        let sprite = commands
            .spawn_bundle(textures.empty().tap_mut(|b| {
                b.transform = Transform {
                    translation: Vec3::new(x as f32 * 32.0, y as f32 * 32.0, 3.0),
                    ..Default::default()
                };
            }))
            .id();

        MineCell {
            sprite,
            state: MineCellState::Empty,
            modified: false,
        }
    }
}

pub fn render_mines(mut q: Query<&mut Minefield>, mut sprites: Query<&mut TextureAtlasSprite>) {
    let mut minefield = q.single_mut();
    let cols = minefield.num_columns();
    for ix in (0..minefield.num_elements()).map( |pos| {
        (pos / cols, pos % cols)
    }) {
        let cell = &mut (**minefield)[ix];
        if cell.modified == true {
            cell.modified = false;
            *sprites.get_mut(cell.sprite).unwrap() = match cell.state {
                MineCellState::Empty | MineCellState::Mine => {
                    // commands.insert(sprite, textures.empty());
                    TextureAtlasSprite::new(9)
                }
                MineCellState::FlaggedMine | MineCellState::FlaggedEmpty => {
                    // commands.insert(sprite, textures.flag());
                    TextureAtlasSprite::new(10)
                }
                MineCellState::FoundEmpty(x) => TextureAtlasSprite::new(x as usize),
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MineCellState {
    Empty,
    Mine,
    FoundEmpty(u8),
    FlaggedEmpty,
    FlaggedMine,
}

#[derive(Component)]
struct Mine(Position);

#[derive(Deref, DerefMut, Component)]
pub struct Minefield(pub(super) Array2D<MineCell>);

impl Minefield {
    pub fn iter_neighbors_enumerated(
        &self,
        pos: Position,
    ) -> impl Iterator<Item = (Position, &MineCell)> + '_ {
        self.iter_neighbor_positions(pos)
            .map(|pos| (pos.clone(), &self[pos]))
    }

    pub fn iter_neighbor_positions(&self, pos: Position) -> impl Iterator<Item = Position> {
        pos.iter_neighbors(self.num_columns() as u32, self.num_rows() as u32)
    }
}

impl Index<Position> for Minefield {
    type Output = MineCell;

    fn index(&self, Position(y, x): Position) -> &Self::Output {
        &(**self)[(x as usize, y as usize)]
    }
}

impl IndexMut<Position> for Minefield {
    fn index_mut(&mut self, Position(y, x): Position) -> &mut Self::Output {
        &mut (**self)[(x as usize, y as usize)]
    }
}
