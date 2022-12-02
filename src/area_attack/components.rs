use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{
    common::Position,
    server::GameBundle,
    singleplayer::minefield::{FieldShape, Minefield},
};

use super::{states::AreaAttackState, AreaAttackServer, AREA_ATTACK_MARKER};

#[derive(Bundle)]
pub struct AreaAttackBundle {
    game: GameBundle,
    field: Minefield,
    template: Handle<FieldShape>,
    state: AreaAttackState,
    typed_marker: AreaAttackServer,
}

impl AreaAttackBundle {
    pub fn new(
        commands: &mut Commands,
        template: Handle<FieldShape>,
        template_set: &Res<Assets<FieldShape>>,
    ) -> Self {
        Self {
            game: GameBundle {
                marker: AREA_ATTACK_MARKER,
            },
            field: Minefield::new_shaped(
                |&position| {
                    commands
                        .spawn(ServerTileBundle {
                            tile: ServerTile::Empty,
                            position,
                        })
                        .id()
                },
                template_set.get(&template).unwrap(),
            ),
            template,
            state: AreaAttackState::Selecting,
            typed_marker: AreaAttackServer,
        }
    }
}

#[derive(Component)]
pub enum ServerTile {
    /// No one has claimed the tile, and the tile does not contain a mine
    Empty,
    /// A player has claimed the tile
    Owned { player: Entity },
    /// There is a mine on this tile, and it has not been revealed
    Mine,
    /// There is a mine on this tile, and it has been revealed
    HardMine,
}

#[derive(Bundle)]
pub struct ServerTileBundle {
    pub tile: ServerTile,
    pub position: Position,
}

#[derive(Component, Serialize, Deserialize)]
pub enum ClientTile {
    /// No one has claimed this tile, and it isn't known whether it is blank or contains a mine
    Unknown,
    /// This tile has been claimed by the player specified by the given ID. In addition, if the
    /// client using this type is the one that owns this tile, it will know the number of cells
    /// neighboring this tile which have mines (`num_neighbors`), which can also be zero, in which
    /// case a filled tile without numbers will be shown. If this tile is not owned by the client,
    /// then this field will always be zero.
    Owned { player: Entity, num_neighbors: u8 },
    /// There is a mine on this tile, and it has been revealed
    HardMine,
}

#[derive(Serialize, Deserialize, Clone, Copy, Component, EnumIter)]
pub enum PlayerColor {
    Yellow,
    Green,
    Blue,
    Purple,
}
