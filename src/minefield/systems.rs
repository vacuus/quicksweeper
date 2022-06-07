use super::field::*;
use crate::common::{CheckCell, FlagCell, GameInitData, InitCheckCell, Position};
use crate::state::SingleplayerState;
use crate::textures::MineTextures;
use array2d::Array2D;
use bevy::prelude::*;
use itertools::Itertools;
use iyes_loopless::prelude::*;
use rand::prelude::{SliceRandom, StdRng};
use std::collections::VecDeque;
use tap::Pipe;

pub fn create_minefield(
    mut commands: Commands,
    textures: Res<MineTextures>,
    init_data: Res<GameInitData>,
) {
    let GameInitData { rows, cols, .. } = *init_data;
    let len = rows * cols;

    let minefield_iter = (0..len).map(|ix| {
        let pos = Position::new(ix % cols, ix / cols);
        let entity = MineCell::new_empty(pos, &textures)
            .pipe(|cell| commands.spawn_bundle(cell))
            .id();
        (pos, entity)
    });

    let minefield = Minefield {
        field: minefield_iter.collect(),
        remaining_blank: len as usize * 8 / 10,
    };
    commands.spawn().insert(minefield);

    commands.insert_resource(NextState(SingleplayerState::PreGame));
}

pub fn generate_minefield(
    mut commands: Commands,
    mut position: EventReader<InitCheckCell>,
    mut write_back: EventWriter<CheckCell>,
    minefield: Query<&Minefield>,
    mut states: Query<&mut MineCellState>,
    mut rng: ResMut<StdRng>,
) {
    if let Some(InitCheckCell(pos)) = position.iter().next().cloned() {
        write_back.send(CheckCell(pos.clone()));

        let minefield = minefield.single();
        // let cols = minefield.num_columns();
        // let len = minefield.num_elements();

        let len = minefield.len();
        let minefield_vec = minefield.iter().collect_vec();
        let neighbors = minefield.iter_neighbor_positions(pos).collect_vec();

        minefield_vec
            .choose_multiple_weighted(&mut *rng, len - minefield.remaining_blank, |(pos, _)| {
                if neighbors.contains(pos) {
                    0.0
                } else {
                    1.0 / (len - neighbors.len()) as f32
                }
            })
            .unwrap()
            .for_each(|(_, cell)| {
                *states.get_mut(**cell).unwrap() = MineCellState::Mine;
            });

        // let neighbors = minefield
        //     .iter_neighbor_positions(pos.clone())
        //     .chain(std::iter::once(pos.clone()))
        //     .map(|pos| pos.y * cols as u32 + pos.x)
        //     .collect_vec();

        // generate mines with density 3/10
        // rand::seq::index::sample_weighted(
        //     &mut *rng,
        //     len,
        //     |pos| {
        //         if neighbors.contains(&(pos as u32)) {
        //             0.0
        //         } else {
        //             1.0 / (len - neighbors.len()) as f32
        //         }
        //     },
        //     len - minefield.remaining_blank,
        // )
        // .unwrap()
        // .into_iter()
        // .map(|x| Position::new((x % cols) as u32, (x / cols) as u32))
        // .for_each(|pos| {
        //     minefield[pos].set_state(MineCellState::Mine);
        // });

        commands.insert_resource(NextState(SingleplayerState::Game));
    }
}

pub fn reveal_cell(
    mut commands: Commands,
    mut field: Query<&mut Minefield>,
    mut states: Query<&mut MineCellState>,
    mut ev: EventReader<CheckCell>,
    mut check_next: Local<VecDeque<Position>>,
) {
    check_next.extend(ev.iter().map(|CheckCell(x)| x).cloned());
    let mut field = field.single_mut();

    while let Some(position) = check_next.pop_front() {
        let neighbors = field
            .iter_neighbors_enumerated(position)
            .map(|(a, b)| (a, b))
            .collect_vec();

        let mut found = None;

        let checking = states.get(field[position]).unwrap();
        match *checking {
            MineCellState::Empty => {
                let count_mine_neighbors = neighbors
                    .iter()
                    .filter(|(_, state)| states.get(*state).unwrap().is_mine())
                    .count() as u8;
                if count_mine_neighbors == 0 {
                    check_next.extend(
                        neighbors
                            .into_iter()
                            .filter(|(_, state)| !states.get(*state).unwrap().is_flagged())
                            .map(|(pos, _)| pos.clone()),
                    );
                }
                found = Some(count_mine_neighbors);
                // *checking = MineCellState::FoundEmpty(count_mine_neighbors);
            }
            MineCellState::Mine => {
                commands.insert_resource(NextState(SingleplayerState::GameFailed));
            }
            MineCellState::FoundEmpty(x) => {
                if neighbors
                    .iter()
                    .filter(|(_, state)| states.get_mut(*state).unwrap().is_flagged())
                    .count()
                    == x as usize
                {
                    check_next.extend(
                        neighbors
                            .into_iter()
                            .filter(|(_, state)| !states.get(*state).unwrap().is_marked())
                            .map(|(pos, _)| pos.clone()),
                    );
                }
            }
            _ => (), // ignore marked cells
        }

        if let Some(num) = found {
            *states.get_mut(field[position]).unwrap() = MineCellState::FoundEmpty(num);
            field.remaining_blank -= 1;
        }

        if field.remaining_blank == 0 {
            commands.insert_resource(NextState(SingleplayerState::GameSuccess));
        } else {
            println!("remaining: {}", field.remaining_blank)
        }
    }
}

pub fn flag_cell(
    mut ev: EventReader<FlagCell>,
    mut field: Query<&mut Minefield>,
    mut states: Query<&mut MineCellState>,
) {
    for FlagCell(pos) in ev.iter() {
        let mut state = states
            .get_mut(field.get_single_mut().unwrap()[*pos])
            .unwrap();
        *state = match *state {
            MineCellState::Empty => MineCellState::FlaggedEmpty,
            MineCellState::FlaggedEmpty => MineCellState::Empty,
            MineCellState::Mine => MineCellState::FlaggedMine,
            MineCellState::FlaggedMine => MineCellState::Mine,
            _ => state.clone(), // ignore revealed cells
                                // TODO: move assignment inside match
        }
    }
}
