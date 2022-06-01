use std::collections::VecDeque;

use crate::common::CheckCell;
use crate::textures::MineTextures;
use crate::AppState;
use array2d::Array2D;
use bevy::prelude::*;
use itertools::Itertools;
use iyes_loopless::prelude::*;
use rand::prelude::StdRng;
use tap::Tap;

mod field;
pub use field::*;

fn create_minefield(
    mut commands: Commands,
    textures: Res<MineTextures>,
) {
    let rows = 10;
    let cols = 20;
    let len = rows * cols;

    let minefield_iter = (0..len).map(|ix| {
        let (y, x) = (ix / cols, ix % cols);

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
        }
    });

    let minefield = Minefield(Array2D::from_iter_row_major(minefield_iter, rows, cols));
    commands.spawn().insert(minefield);
    
    commands.insert_resource(NextState(AppState::PreGame));
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
    mut commands: Commands,
    mut field: Query<&mut Minefield>,
    mut cell_sprite: Query<&mut TextureAtlasSprite>,
    mut ev: EventReader<CheckCell>,
    mut check_next: Local<VecDeque<CheckCell>>,
) {
    check_next.extend(ev.iter().cloned());
    let mut field = field.single_mut();

    if let Some(CheckCell(position)) = check_next.pop_front() {
        println!("Event received with position {position:?}");

        let (mine_neighbors, blank_neighbors): (Vec<_>, Vec<_>) = field
            .iter_neighbors_enumerated(position.clone())
            .map(|(a, b)| (a, b.clone()))
            .partition(|(_, x)| matches!(x.state, MineCellState::Mine | MineCellState::Flagged));

        let checking = &mut field[position.clone()];

        match checking.state {
            MineCellState::Empty => {
                if mine_neighbors.is_empty() {
                    check_next.extend(blank_neighbors.into_iter().map(|(pos, _)| CheckCell(pos)));
                }
                checking.state = MineCellState::FoundEmpty(mine_neighbors.len() as u8);
                *cell_sprite.get_mut(checking.sprite).unwrap() =
                    TextureAtlasSprite::new(mine_neighbors.len());
            }
            MineCellState::Mine => {
                println!("end game");
                commands.insert_resource(NextState(AppState::GameFailed));
            }
            MineCellState::FoundEmpty(_) => {
                println!("reveal if filled");
            }
            _ => (), // ignore marked cells
        }
    }
}

pub struct MinefieldPlugin;

impl Plugin for MinefieldPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_enter_system(AppState::Game, generate_minefield)
            .add_system(create_minefield.run_in_state(AppState::Loading))
            .add_system(reveal_cell.run_in_state(AppState::Game));
    }
}
