use bevy::{prelude::*, utils::Uuid};

mod cell;
mod server_systems;

use server_systems::*;

use crate::{
    registry::GameRegistry,
    server::{GameBundle, GameDescriptor, GameMarker},
    singleplayer::minefield::Minefield,
};

pub const AREA_ATTACK_UUID: Uuid = match Uuid::try_parse("040784a0-e905-44a9-b698-14a71a29b3fd") {
    Ok(val) => val,
    Err(_) => unreachable!(),
};

#[derive(Component)]
pub struct AreaAttackServer;

impl Plugin for AreaAttackServer {
    fn build(&self, app: &mut App) {
        app.add_startup_system(|mut registry: ResMut<GameRegistry>| {
            registry.insert(
                GameMarker(AREA_ATTACK_UUID),
                GameDescriptor {
                    name: "Area Attack".to_string(),
                    description: "Race to claim the board for yourself".to_string(),
                },
            );
        })
        .add_system(create_game);
    }
}

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
            },
            field,
        }
    }
}
