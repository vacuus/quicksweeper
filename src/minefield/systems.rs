use super::field::*;
use crate::common::{CheckCell, FlagCell, GameInitData, InitCheckCell, Position};
use crate::state::AppState;
use crate::textures::MineTextures;
use array2d::Array2D;
use bevy::prelude::*;
use itertools::Itertools;
use iyes_loopless::prelude::*;
use rand::prelude::StdRng;
use std::collections::VecDeque;

pub fn create_minefield(
    mut commands: Commands,
    textures: Res<MineTextures>,
    init_data: Res<GameInitData>,
) {
    let GameInitData { rows, cols, .. } = *init_data;
    let len = rows * cols;

    let minefield_iter = (0..len).map(|ix| {
        let (x, y) = (ix % cols, ix / cols);
        MineCell::new_empty(&mut commands, Position::new(x as u32, y as u32), &textures)
    });

    let minefield = Minefield {
        field: Array2D::from_iter_row_major(minefield_iter, rows as usize, cols as usize),
        remaining_blank: len as usize * 8 / 10,
    };
    commands.spawn().insert(minefield);

    commands.insert_resource(NextState(AppState::PreGame));
}

pub fn generate_minefield(
    mut commands: Commands,
    mut position: EventReader<InitCheckCell>,
    mut write_back: EventWriter<CheckCell>,
    mut minefield: Query<&mut Minefield>,
    mut rng: ResMut<StdRng>,
) {
    if let Some(InitCheckCell(pos)) = position.iter().next().cloned() {
        write_back.send(CheckCell(pos.clone()));

        let mut minefield = minefield.single_mut();
        let cols = minefield.num_columns();
        let len = minefield.num_elements();

        let neighbors = minefield
            .iter_neighbor_positions(pos.clone())
            .chain(std::iter::once(pos.clone()))
            .map(|pos| pos.y * cols as u32 + pos.x)
            .collect_vec();

        // generate mines with density 3/10
        rand::seq::index::sample_weighted(
            &mut *rng,
            len,
            |pos| {
                if neighbors.contains(&(pos as u32)) {
                    0.0
                } else {
                    1.0 / (len - neighbors.len()) as f32
                }
            },
            len - minefield.remaining_blank,
        )
        .unwrap()
        .into_iter()
        .map(|x| Position::new((x % cols) as u32, (x / cols) as u32))
        .for_each(|pos| {
            minefield[pos].set_state(MineCellState::Mine);
        });

        commands.insert_resource(NextState(AppState::Game));
    }
}

pub fn reveal_cell(
    mut commands: Commands,
    mut field: Query<&mut Minefield>,
    mut ev: EventReader<CheckCell>,
    mut check_next: Local<VecDeque<Position>>,
) {
    check_next.extend(ev.iter().map(|CheckCell(x)| x).cloned());
    let mut field = field.single_mut();

    while let Some(position) = check_next.pop_front() {
        let neighbors = field
            .iter_neighbors_enumerated(position.clone())
            .map(|(a, b)| (a, b.clone()))
            .collect_vec();

        use MineCellState::*;

        let mine_neighbors = || {
            neighbors
                .iter()
                .filter(|(_, cell)| matches!(cell.state(), Mine | FlaggedMine))
        };

        let unflagged_neighbors = || {
            neighbors
                .iter()
                .filter(|(_, cell)| !matches!(cell.state(), FlaggedMine | FlaggedEmpty))
        };

        let flagged_neighbors = || {
            neighbors
                .iter()
                .filter(|(_, cell)| matches!(cell.state(), FlaggedMine | FlaggedEmpty))
        };

        let unmarked = || {
            neighbors
                .iter()
                .filter(|(_, cell)| matches!(cell.state(), Mine | Empty))
        };

        let mut found = false;

        let checking = &mut field[position.clone()];
        match checking.state() {
            MineCellState::Empty => {
                let count_mine_neighbors = mine_neighbors().count() as u8;
                if count_mine_neighbors == 0 {
                    check_next.extend(unflagged_neighbors().map(|(pos, _)| pos.clone()));
                }
                found = true;
                checking.set_state(MineCellState::FoundEmpty(count_mine_neighbors));
            }
            MineCellState::Mine => {
                commands.insert_resource(NextState(AppState::GameFailed));
            }
            MineCellState::FoundEmpty(x) => {
                if flagged_neighbors().count() == *x as usize {
                    check_next.extend(unmarked().map(|(pos, _)| pos.clone()));
                }
            }
            _ => (), // ignore marked cells
        }

        if found {
            field.remaining_blank -= 1;
        }

        if dbg!(field.remaining_blank) == 0 {
            println!("Success!");
            commands.insert_resource(NextState(AppState::GameSuccess));
        }
    }
}

pub fn flag_cell(mut ev: EventReader<FlagCell>, mut field: Query<&mut Minefield>) {
    for FlagCell(pos) in ev.iter() {
        println!("flag event received with position {pos:?}");
        let cell = &mut field.get_single_mut().unwrap()[pos.clone()];
        match cell.state() {
            MineCellState::Empty => cell.set_state(MineCellState::FlaggedEmpty),
            MineCellState::FlaggedEmpty => cell.set_state(MineCellState::Empty),
            MineCellState::Mine => cell.set_state(MineCellState::FlaggedMine),
            MineCellState::FlaggedMine => cell.set_state(MineCellState::Mine),
            _ => (), // ignore marked cells
        }
    }
}
