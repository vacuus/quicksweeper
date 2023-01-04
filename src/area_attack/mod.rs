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
    area_attack::{components::FreezeTimer, protocol::AreaAttackUpdate},
    main_menu::{Menu, ToGame},
    registry::GameRegistry,
    server::{GameDescriptor, GameMarker, LocalEvent},
};

use self::{
    components::{RevealTile, SendTile},
    protocol::AreaAttackRequest,
    puppet::PuppetTable,
    states::AreaAttack,
};

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
            .add_event::<SendTile>()
            .add_event::<RevealTile>()
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(Menu::Loading)
                    .with_system(broadcast_positions)
                    .with_system(create_game)
                    .with_system(reveal_tiles)
                    .with_system(unmark_init_access)
                    .with_system(prepare_player)
                    .with_system(unfreeze_players)
                    .with_system(net_events)
                    .with_system(selection_transition)
                    .with_system(update_selecting_tile)
                    .with_system(update_stage1_tile)
                    .into(),
            );
    }
}

pub struct AreaAttackClient;

impl Plugin for AreaAttackClient {
    fn build(&self, app: &mut App) {
        use AreaAttack::*;
        app.add_loopless_state(Inactive)
            .init_resource::<PuppetTable>()
            .init_resource::<FreezeTimer>()
            .add_event::<AreaAttackUpdate>()
            .add_startup_system(registry_entry)
            .add_system(|mut commands: Commands, mut ev: EventReader<ToGame>| {
                if ev.iter().any(|e| **e == AREA_ATTACK_MARKER) {
                    // transition from menu into game
                    commands.insert_resource(NextState(Selecting))
                }
            })
            .add_system(client_systems::begin_game.run_in_state(Selecting))
            .add_system(puppet::update_cursor_colors)
            .add_exit_system(Menu::Loading, client_systems::create_freeze_timer)
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(Inactive)
                    .with_system(client_systems::send_position)
                    .with_system(client_systems::request_reveal)
                    .with_system(client_systems::draw_tiles)
                    .with_system(client_systems::freeze_timer)
                    // Systems for receiving network events
                    .with_system(client_systems::listen_net)
                    .with_system(client_systems::reset_field)
                    .with_system(client_systems::player_update)
                    .with_system(client_systems::self_update)
                    .with_system(client_systems::puppet_control)
                    .with_system(client_systems::state_transitions)
                    .into(),
            );
    }
}
