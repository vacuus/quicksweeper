use bevy::{prelude::*, utils::Uuid};

use crate::{
    server::{GameBundle, GameDescriptor, GameMarker},
    singleplayer::minefield::Minefield,
};

const AREA_ATTACK_UUID: Uuid = match Uuid::try_parse("040784a0-e905-44a9-b698-14a71a29b3fd") {
    Ok(val) => val,
    Err(_) => unreachable!(),
};

#[derive(Component)]
struct AreaAttack;

#[derive(Bundle)]
struct AreaAttackBundle {
    game: GameBundle,
    field: Minefield,
}

impl AreaAttackBundle {
    fn new(field: Minefield) -> Self {
        Self {
            game: GameBundle {
                marker: GameMarker(AREA_ATTACK_UUID),
                descriptor: GameDescriptor {
                    name: "Area Attack".to_string(),
                    description: "Race to claim the board for yourself".to_string(),
                },
                players: default(),
            },
            field,
        }
    }
}
