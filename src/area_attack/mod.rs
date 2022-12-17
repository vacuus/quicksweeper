use bevy::{prelude::*, utils::Uuid};

mod client_systems;
mod components;
mod protocol;
pub mod puppet;
mod server_systems;
mod states;

use iyes_loopless::prelude::*;
use server_systems::*;

use crate::{
    main_menu::{MenuState, ToGame},
    registry::GameRegistry,
    server::{GameDescriptor, GameMarker, LocalEvent},
};

use self::{protocol::AreaAttackRequest, puppet::PuppetTable, states::AreaAttackState, components::ModifyTile};

pub const AREA_ATTACK_MARKER: GameMarker = GameMarker(
    match Uuid::try_parse("040784a0-e905-44a9-b698-14a71a29b3fd") {
        Ok(val) => val,
        Err(_) => unreachable!(),
    },
);

fn registry_entry(mut registry: ResMut<GameRegistry>) {
    registry.insert(
        AREA_ATTACK_MARKER,
        GameDescriptor {
            name: "Area Attack".to_string(),
            description: "Race to claim the board for yourself".to_string(),
        },
    );
}

#[derive(Component)]
pub struct AreaAttackServer;

impl Plugin for AreaAttackServer {
    fn build(&self, app: &mut App) {
        app.add_startup_system(registry_entry)
            .add_event::<LocalEvent<AreaAttackRequest>>()
            .add_event::<ModifyTile>()
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(MenuState::Loading)
                    .with_system(broadcast_positions)
                    .with_system(create_game)
                    .with_system(send_tiles)
                    .with_system(unmark_init_access)
                    .with_system(prepare_player)
                    .with_system(net_events)
                    .with_system(selection_transition)
                    .with_system(update_selecting_tile)
                    .into(),
            );
    }
}

pub struct AreaAttackClient;

impl Plugin for AreaAttackClient {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(AreaAttackState::Inactive)
            .init_resource::<PuppetTable>()
            .add_startup_system(registry_entry)
            .add_system(|mut commands: Commands, mut ev: EventReader<ToGame>| {
                if ev.iter().any(|e| **e == AREA_ATTACK_MARKER) {
                    // transition from menu into game
                    commands.insert_resource(NextState(AreaAttackState::Selecting))
                }
            })
            .add_system(client_systems::begin_game.run_in_state(AreaAttackState::Selecting))
            .add_system(puppet::update_cursor_colors)
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(AreaAttackState::Inactive)
                    .with_system(client_systems::listen_net)
                    .with_system(client_systems::send_position)
                    .with_system(client_systems::request_reveal)
                    .with_system(client_systems::draw_mines)
                    .into(),
            );
    }
}
