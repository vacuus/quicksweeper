use super::{field::*, GameOutcome};
use crate::{
    common::{CheckCell, FlagCell, InitCheckCell},
    cursor::CursorPosition,
};
use bevy::prelude::*;
use itertools::Itertools;
use rand::prelude::SliceRandom;
use std::collections::VecDeque;

pub fn destroy_minefields(
    mut commands: Commands,
    minefield: Query<Entity, With<Minefield>>,
    states: Query<Entity, With<MineCellState>>,
) {
    minefield.for_each(|map| commands.entity(map).despawn());
    states.for_each(|ent| commands.entity(ent).despawn());
}

pub fn wipe_minefields(mut states: Query<&mut MineCellState>) {
    states.for_each_mut(|mut state| *state = MineCellState::Empty)
}

pub fn generate_minefield(
    mut check: EventReader<InitCheckCell>,
    mut write_back: EventWriter<CheckCell>,
    minefields: Query<(Entity, &Minefield)>,
    mut states: Query<&mut MineCellState>,
) {
    if let Some(ev) = check.iter().next().map(|x| x.clone()) {
        let exclude = ev.positions;
        for position in exclude.iter() {
            // TODO: Synchronize with system `check_cell`
            write_back.send(CheckCell(CursorPosition(*position, ev.minefield)));
        }

        minefields
            .iter()
            .find(|(field, _)| *field == ev.minefield)
            .map(|(_, field)| {
                let minefield_vec = field
                    .occupied_entries()
                    .filter_map(|(a, b)| b.map(|b| (a, b)))
                    .collect_vec();

                minefield_vec
                    .choose_multiple_weighted(
                        &mut rand::thread_rng(),
                        minefield_vec.len() - field.remaining_blank,
                        |&(&pos, _)| {
                            if exclude.contains(&pos.into()) {
                                0.0
                            } else {
                                1.0 / (minefield_vec.len() - exclude.len()) as f32
                            }
                        },
                    )
                    .unwrap()
                    .for_each(|&(_, cell)| {
                        *states.get_mut(cell).unwrap() = MineCellState::Mine;
                    });
            });
    }
}

pub fn reveal_cell(
    mut fields: Query<&mut Minefield>,
    mut states: Query<&mut MineCellState>,
    mut ev: EventReader<CheckCell>,
    mut check_next: Local<VecDeque<CursorPosition>>,
    mut finish_state: EventWriter<GameOutcome>,
) {
    check_next.extend(ev.iter().map(|CheckCell(pos)| pos.clone()));

    while let Some(CursorPosition(position, ent)) = check_next.pop_front() {
        let mut field = fields.get_mut(ent).unwrap();

        let neighbors = field
            .iter_neighbors_enumerated(position)
            .map(|(a, b)| (a, states.get(b).unwrap().clone()))
            .collect_vec();

        let mut checking = states.get_mut(field[&position]).unwrap();
        match *checking {
            MineCellState::Empty => {
                let count_mine_neighbors = neighbors
                    .iter()
                    .filter(|(_, state)| state.is_mine())
                    .count() as u8;
                if count_mine_neighbors == 0 {
                    check_next.extend(neighbors.into_iter().filter_map(|(pos, state)| {
                        (!state.is_flagged()).then(|| CursorPosition(pos, ent))
                    }));
                }

                *checking = MineCellState::Revealed(count_mine_neighbors);
                field.remaining_blank -= 1;
            }
            MineCellState::Mine => {
                finish_state.send(GameOutcome::Failed);
            }
            MineCellState::Revealed(x) => {
                if neighbors
                    .iter()
                    .filter(|(_, state)| state.is_flagged())
                    .count()
                    == x as usize
                {
                    check_next.extend(neighbors.into_iter().filter_map(|(pos, state)| {
                        (!state.is_marked()).then(|| CursorPosition(pos, ent))
                    }));
                }
            }
            _ => (), // ignore marked cells
        }

        if field.remaining_blank == 0 {
            finish_state.send(GameOutcome::Succeeded);
        }
    }
}

pub fn flag_cell(
    mut ev: EventReader<FlagCell>,
    mut fields: Query<&mut Minefield>,
    mut states: Query<&mut MineCellState>,
) {
    for FlagCell(CursorPosition(pos, field)) in ev.iter() {
        let mut state = states
            .get_mut(fields.get_mut(*field).unwrap()[pos])
            .unwrap();
        match *state {
            MineCellState::Empty => Some(MineCellState::FlaggedEmpty),
            MineCellState::FlaggedEmpty => Some(MineCellState::Empty),
            MineCellState::Mine => Some(MineCellState::FlaggedMine),
            MineCellState::FlaggedMine => Some(MineCellState::Mine),
            _ => None, // ignore revealed cells
        }
        .into_iter()
        .for_each(|x| *state = x);
    }
}

pub fn display_mines(mut cells: Query<(&mut TextureAtlasSprite, &MineCellState)>) {
    cells.for_each_mut(|(mut sprite, state)| {
        if *state == MineCellState::Mine {
            *sprite = TextureAtlasSprite::new(11)
        }
    });
}
