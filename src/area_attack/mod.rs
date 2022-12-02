use bevy::{prelude::*, utils::Uuid};

mod protocol;
mod server_systems;
mod states;
mod tile;

use iyes_loopless::prelude::*;
use server_systems::*;

use crate::{
    main_menu::MenuState,
    registry::GameRegistry,
    server::{GameBundle, GameDescriptor, GameMarker},
    singleplayer::minefield::{FieldShape, Minefield},
};

use self::{
    states::AreaAttackState,
    tile::{ServerTile, ServerTileBundle},
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

#[derive(Bundle)]
struct AreaAttackBundle {
    game: GameBundle,
    field: Minefield,
    template: Handle<FieldShape>,
    state: AreaAttackState,
    typed_marker: AreaAttackServer,
}

impl AreaAttackBundle {
    fn new(
        commands: &mut Commands,
        template: Handle<FieldShape>,
        template_set: &Res<Assets<FieldShape>>,
    ) -> Self {
        Self {
            game: GameBundle {
                marker: GameMarker(AREA_ATTACK_UUID),
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
