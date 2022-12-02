use bevy::{prelude::*, utils::Uuid};

mod components;
mod protocol;
mod server_systems;
mod states;

use iyes_loopless::prelude::*;
use server_systems::*;

use crate::{
    main_menu::MenuState,
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
                .into(),
        );
    }
}

pub struct AreaAttackClient;

impl Plugin for AreaAttackClient {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(AreaAttackState::Inactive);
    }
}
