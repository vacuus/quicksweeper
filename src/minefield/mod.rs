use std::ops::{Index, IndexMut};

use crate::common::{CheckCell, Position};
use crate::textures::MineTextures;
use crate::AppState;
use array2d::Array2D;
use bevy::prelude::*;
use derive_more::Deref;
use itertools::Itertools;
use iyes_loopless::prelude::*;
use rand::prelude::StdRng;
use tap::Tap;

#[derive(Clone, Debug)]
pub struct MineCell {
    sprite: Entity,
    state: MineCellState,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MineCellState {
    Empty,
    Mine,
    MarkedEmpty(u8),
    MarkedMine,
}

#[derive(Component)]
struct Mine(Position);

#[derive(Deref, DerefMut, Component)]
pub struct Minefield(Array2D<MineCell>);

impl Index<Position> for Minefield {
    type Output = MineCell;

    fn index(&self, Position(x, y): Position) -> &Self::Output {
        &(**self)[(x, y)]
    }
}

impl IndexMut<Position> for Minefield {
    fn index_mut(&mut self, Position(x, y): Position) -> &mut Self::Output {
        &mut (**self)[(x, y)]
    }
}

fn generate_minefield(
    mut commands: Commands,
    textures: Res<MineTextures>,
    mut rng: ResMut<StdRng>,
) {
    let rows = 10;
    let cols = 20;

    let len = rows * cols;

    // generate mines with density 3/10
    let indices = rand::seq::index::sample(&mut *rng, len, len * 3 / 10);

    let minefield_iter = (0..len).map(|ix| {
        let (y, x) = (ix / cols, ix % cols);

        let state = indices
            .iter()
            .contains(&ix)
            .then(|| MineCellState::Mine)
            .unwrap_or(MineCellState::Empty);

        let sprite = commands
            .spawn_bundle(textures.empty().tap_mut(|b| {
                b.transform = Transform {
                    translation: Vec3::new(x as f32 * 32.0, y as f32 * 32.0, 3.0),
                    ..Default::default()
                };
            }))
            .id();

        MineCell { sprite, state }
    });

    let minefield = Array2D::from_iter_row_major(minefield_iter, rows, cols);

    commands.spawn().insert(Minefield(minefield));
}

fn reveal_cell(
    mut field: Query<&mut Minefield>,
    mut ev: EventReader<CheckCell>,
    mut feedback: EventWriter<CheckCell>,
) {
    let field = field.single_mut();
    ev.iter().for_each(|CheckCell(position)| {
        println!("Event received with position {position:?}");


        match field[position.clone()].state {
            MineCellState::Empty => {
                
            }
            MineCellState::Mine => {
                println!("end game")
            }
            MineCellState::MarkedEmpty(_) => {
                println!("reveal if filled");
            }
            _ => (), // ignore marked cells
        }

        //TODO
    })
}

pub struct MinefieldPlugin;

impl Plugin for MinefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::Game, generate_minefield)
            .add_system(reveal_cell.run_in_state(AppState::Game));
    }
}
