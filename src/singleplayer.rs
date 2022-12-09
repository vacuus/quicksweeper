use crate::{
    common::{InitCheckCell, Position},
    cursor::*,
    load::{Field, MineTextures, Textures},
};
use bevy::prelude::*;
use iyes_loopless::{
    prelude::{AppLooplessStateExt, IntoConditionalSystem},
    state::NextState,
};
use minefield::{systems::*, FieldShape, GameOutcome, Minefield};

use self::minefield::specific::{MineCell, CELL_SIZE};

mod menu;
pub mod minefield;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum SingleplayerState {
    Inactive,
    PreGame,
    Game,
    GameFailed,
    GameSuccess,
}

fn advance_to_end(mut commands: Commands, mut game_outcome: EventReader<GameOutcome>) {
    if let Some(outcome) = game_outcome.iter().next() {
        match outcome {
            GameOutcome::Failed => {
                commands.insert_resource(NextState(SingleplayerState::GameFailed))
            }
            GameOutcome::Succeeded => {
                commands.insert_resource(NextState(SingleplayerState::GameSuccess))
            }
        }
    }
}

fn advance_to_game(mut commands: Commands, init_move: EventReader<InitCheckCell>) {
    if !init_move.is_empty() {
        commands.insert_resource(NextState(SingleplayerState::Game))
    }
}

fn create_entities(
    mut commands: Commands,
    field_templates: Res<Assets<FieldShape>>,
    template_handles: Res<Field>,
    mine_textures: Res<MineTextures>,
    textures: Res<Textures>,
) {
    // create minefield
    let field_template = field_templates
        .get(template_handles.take_one(&mut rand::thread_rng()))
        .unwrap();
    let minefield = Minefield::new_shaped(
        |&pos| {
            commands
                .spawn(MineCell::new_empty(pos, &mine_textures))
                .id()
        },
        field_template,
    );
    let minefield_entity = commands.spawn(()).insert(minefield).id();

    // get center of minefield
    let init_position = field_template.center().unwrap_or(Position { x: 0, y: 0 });

    // create cursor
    commands.spawn(CursorBundle {
        cursor: Cursor::new(minefield_entity),
        position: init_position,
        texture: SpriteBundle {
            texture: textures.cursor.clone(),
            transform: Transform {
                translation: init_position.absolute(CELL_SIZE, CELL_SIZE).extend(3.0),
                ..default()
            },
            ..default()
        },
    });
}

pub struct SingleplayerMode;

impl Plugin for SingleplayerMode {
    fn build(&self, app: &mut App) {
        use SingleplayerState::*;
        app
            // state
            .add_loopless_state(Inactive)
            // menu after game complete
            .add_plugin(menu::MenuPlugin)
            // state change startup and cleanup
            .add_exit_system(Inactive, create_entities)
            .add_system(generate_minefield.run_in_state(PreGame))
            .add_exit_system(GameFailed, wipe_minefields)
            .add_exit_system(GameSuccess, wipe_minefields)
            .add_system(advance_to_game.run_in_state(PreGame))
            .add_system(advance_to_end.run_in_state(Game))
            .add_enter_system(GameFailed, display_mines)
            // logic for leaving game
            .add_enter_system(Inactive, destroy_cursors)
            .add_enter_system(Inactive, destroy_minefields)
            // in-game logic
            .add_system(flag_cell.run_in_state(Game))
            .add_system(reveal_cell.run_in_state(Game))
            .add_system(init_check_cell.run_in_state(PreGame))
            .add_system(check_cell.run_in_state(Game));
    }
}
