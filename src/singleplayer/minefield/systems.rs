use super::{field::*, specific::MineCellState, GameOutcome};
use crate::{
    common::{CheckCell, FlagCell, InitCheckCell},
    cursor::CursorPosition,
};
use bevy::prelude::*;
use itertools::Itertools;
use std::collections::VecDeque;

pub fn destroy_minefields(
    mut commands: Commands,
    minefield: Query<Entity, With<Minefield>>,
    states: Query<Entity, With<MineCellState>>,
) {
    minefield.for_each(|map| commands.entity(map).despawn());
    states.for_each(|ent| commands.entity(ent).despawn());
}

pub fn wipe_minefields(
    mut states: Query<&mut MineCellState>,
    mut minefield: Query<&mut Minefield>,
) {
    minefield.for_each_mut(|mut field| {
        states.for_each_mut(|mut state| *state = MineCellState::Empty);

        let amnt_cells = field.occupied_entries().count();
        field.remaining_blank = amnt_cells * 8 / 10;
    })
}

pub fn generate_minefield(
    mut check: EventReader<InitCheckCell>,
    mut write_back: EventWriter<CheckCell>,
    minefields: Query<(Entity, &Minefield)>,
    mut states: Query<&mut MineCellState>,
) {
    if let Some(ev) = check.iter().next().cloned() {
        let exclude = ev.positions;
        for position in exclude.iter() {
            write_back.send(CheckCell(CursorPosition(*position, ev.minefield)));
        }

        if let Some((_, field)) = minefields.iter().find(|(field, _)| *field == ev.minefield) {
            field
                .choose_multiple(&exclude, &mut rand::thread_rng())
                .into_iter()
                .for_each(|(_, cell)| {
                    *states.get_mut(cell).unwrap() = MineCellState::Mine;
                });
        }
    }
}

/// Reacts to requests to reveal cells, and is the sole consumer of [CheckCell] events.
pub fn reveal_cell(
    mut fields: Query<&mut Minefield>,
    mut states: Query<&mut MineCellState>,
    mut ev: ResMut<Events<CheckCell>>,
    mut check_next: Local<VecDeque<CursorPosition>>,
    mut finish_state: EventWriter<GameOutcome>,
) {
    check_next.extend(ev.drain().map(|CheckCell(pos)| pos));

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
                        (!state.is_flagged()).then_some(CursorPosition(pos, ent))
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
                        (!state.is_marked()).then_some(CursorPosition(pos, ent))
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
    use MineCellState::*;
    for FlagCell(CursorPosition(pos, field)) in ev.iter() {
        let mut state = states
            .get_mut(fields.get_mut(*field).unwrap()[pos])
            .unwrap();
        let new_state = match *state {
            Empty => FlaggedEmpty,
            FlaggedEmpty => Empty,
            Mine => FlaggedMine,
            FlaggedMine => Mine,
            Revealed(_) => continue, // ignore revealed cells
        };
        *state = new_state;
    }
}

pub fn display_mines(mut cells: Query<(&mut TextureAtlasSprite, &MineCellState)>) {
    cells.for_each_mut(|(mut sprite, state)| {
        if *state == MineCellState::Mine {
            *sprite = TextureAtlasSprite::new(11)
        }
    });
}
