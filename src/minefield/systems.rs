use super::load::BlankField;
use super::{field::*, GameOutcome};
use crate::common::{CheckCell, CurrentMinefield, FlagCell, InitCheckCell, Position};
use crate::load::{Field, MineTextures};
// use crate::state::SingleplayerState;
use bevy::prelude::*;
use itertools::Itertools;
// use iyes_loopless::prelude::*;
use rand::prelude::SliceRandom;
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
            let entity = MineCell::new_empty(*pos, &textures)
                .pipe(|cell| commands.spawn_bundle(cell))
                .id();
            (*pos, entity)
        });

        let minefield = Minefield {
            field: minefield_iter.collect(),
            remaining_blank: field.len() * 8 / 10,
        };

        let ent = commands.spawn().insert(minefield).id();
        commands.insert_resource(CurrentMinefield(ent));
    }
}

pub fn destroy_minefields(
    mut commands: Commands,
    minefield: Query<Entity, With<Minefield>>,
    states: Query<Entity, With<MineCellState>>,
) {
    minefield.for_each(|map| commands.entity(map).despawn());
    states.for_each(|ent| commands.entity(ent).despawn());
}

pub fn generate_minefield(
    mut position: EventReader<InitCheckCell>,
    mut write_back: EventWriter<CheckCell>,
    minefield: Query<&Minefield>,
    mut states: Query<&mut MineCellState>,
) {
    for InitCheckCell(pos, field) in position.iter() {
        write_back.send(CheckCell(*pos, *field));

        let minefield = minefield.get(*field).unwrap();

        let minefield_vec = minefield.iter().collect_vec();
        let neighbors = minefield
            .iter_neighbor_positions(*pos)
            .chain(std::iter::once(*pos))
            .collect_vec();

        minefield_vec
            .choose_multiple_weighted(
                &mut rand::thread_rng(),
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
    }
}

pub fn reveal_cell(
    mut fields: Query<&mut Minefield>,
    mut states: Query<&mut MineCellState>,
    mut ev: EventReader<CheckCell>,
    mut check_next: Local<VecDeque<(Position, Entity)>>,
    mut finish_state: EventWriter<GameOutcome>,
) {
    check_next.extend(ev.iter().map(|CheckCell(pos, ent)| (*pos, *ent)));

    while let Some((position, ent)) = check_next.pop_front() {
        let mut field = fields.get_mut(ent).unwrap();

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
                            .filter_map(|(pos, state)| (!state.is_flagged()).then(|| (pos, ent))),
                    );
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
                    check_next.extend(
                        neighbors
                            .into_iter()
                            .filter_map(|(pos, state)| (!state.is_marked()).then(|| (pos, ent))),
                    );
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
    for FlagCell(pos, field) in ev.iter() {
        let mut state = states
            .get_mut(fields.get_mut(*field).unwrap()[*pos])
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
