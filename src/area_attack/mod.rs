use bevy::{prelude::*, utils::Uuid};

mod client_systems;
mod components;
mod protocol;
mod server_systems;
mod states;

use iyes_loopless::prelude::*;
use server_systems::*;

use crate::{
    cursor::{pointer_cursor, track_cursor, translate_cursor},
    main_menu::{MenuState, ToGame},
    registry::GameRegistry,
    server::{GameDescriptor, GameMarker},
};

use self::states::AreaAttackState;

pub const AREA_ATTACK_MARKER: GameMarker = GameMarker(
    match Uuid::try_parse("040784a0-e905-44a9-b698-14a71a29b3fd") {
        Ok(val) => val,
        Err(_) => unreachable!(),
    },
);

#[derive(Component)]
pub struct AreaAttackServer;

impl Plugin for AreaAttackServer {
    fn build(&self, app: &mut App) {
        app.add_startup_system(|mut registry: ResMut<GameRegistry>| {
            registry.insert(
                AREA_ATTACK_MARKER,
                GameDescriptor {
                    name: "Area Attack".to_string(),
                    description: "Race to claim the board for yourself".to_string(),
                },
            );
        })
        .add_system_set(
            ConditionSet::new()
                .run_not_in_state(MenuState::Loading)
                .with_system(create_game)
                .with_system(unmark_init_access)
                .with_system(prepare_player)
                .into(),
        );
    }
}

pub struct AreaAttackClient;

impl Plugin for AreaAttackClient {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(AreaAttackState::Inactive)
            .add_system(|mut commands: Commands, mut ev: EventReader<ToGame>| {
                if ev.iter().any(|e| **e == AREA_ATTACK_MARKER) {
                    // transition from menu into game
                    commands.insert_resource(NextState(AreaAttackState::Selecting))
                }
            })
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(AreaAttackState::Inactive)
                    .with_system(client_systems::listen_events)
                    .into(),
            );
    }
}
