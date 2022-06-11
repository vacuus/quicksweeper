use super::field::*;
use super::load::BlankField;
use crate::common::{CheckCell, FlagCell, InitCheckCell, Position};
use crate::load::{MineTextures, Field};
use crate::state::SingleplayerState;
use bevy::prelude::*;
use itertools::Itertools;
use iyes_loopless::prelude::*;
use rand::prelude::{SliceRandom, StdRng};
use std::collections::VecDeque;
use tap::Pipe;

pub fn create_minefield(
    mut commands: Commands,
    textures: Res<MineTextures>,
    field: Res<Field>,
    assets: Res<Assets<BlankField>>,
) {
    if let Some(field) = assets.get(field.field.clone()) {
        let minefield_iter = field.iter().map(|pos| {
            let entity = MineCell::new_empty(pos.clone(), &textures)
                .pipe(|cell| commands.spawn_bundle(cell))
                .id();
            (pos.clone(), entity)
        });

        let minefield = Minefield {
            field: minefield_iter.collect(),
            remaining_blank: field.len() * 8 / 10,
        };
        commands.spawn().insert(minefield);

    }
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

        let minefield_vec = minefield.iter().collect_vec();
        let neighbors = minefield.iter_neighbor_positions(pos).collect_vec();

        minefield_vec
            .choose_multiple_weighted(
                &mut *rng,
                minefield.len() - minefield.remaining_blank,
                |(pos, _)| {
                    if neighbors.contains(pos) {
                        0.0
                    } else {
                        1.0 / (minefield.len() - neighbors.len()) as f32
                    }
                },
            )
            .unwrap()
            .for_each(|(_, cell)| {
                *states.get_mut(**cell).unwrap() = MineCellState::Mine;
            });

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
            .map(|(a, b)| (a, states.get(b).unwrap().clone()))
            .collect_vec();

        let mut checking = states.get_mut(field[position]).unwrap();
        match *checking {
            MineCellState::Empty => {
                let count_mine_neighbors = neighbors
                    .iter()
                    .filter(|(_, state)| state.is_mine())
                    .count() as u8;
                if count_mine_neighbors == 0 {
                    check_next.extend(
                        neighbors
                            .into_iter()
                            .filter(|(_, state)| !state.is_flagged())
                            .map(|(pos, _)| pos.clone()),
                    );
                }
                *checking = MineCellState::FoundEmpty(count_mine_neighbors);

                field.remaining_blank -= 1;
            }
            MineCellState::Mine => {
                commands.insert_resource(NextState(SingleplayerState::GameFailed));
            }
            MineCellState::FoundEmpty(x) => {
                if neighbors
                    .iter()
                    .filter(|(_, state)| state.is_flagged())
                    .count()
                    == x as usize
                {
                    check_next.extend(
                        neighbors
                            .into_iter()
                            .filter(|(_, state)| !state.is_marked())
                            .map(|(pos, _)| pos.clone()),
                    );
                }
            }
            _ => (), // ignore marked cells
        }

        if field.remaining_blank == 0 {
            commands.insert_resource(NextState(SingleplayerState::GameSuccess));
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
        match *state {
            MineCellState::Empty => Some(MineCellState::FlaggedEmpty),
            MineCellState::FlaggedEmpty => Some(MineCellState::Empty),
            MineCellState::Mine => Some(MineCellState::FlaggedMine),
            MineCellState::FlaggedMine => Some(MineCellState::Mine),
            _ => None, // ignore revealed cells
        }
        .map(|x| *state = x);
    }
}
