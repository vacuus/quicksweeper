use crate::{
    common::{InitCheckCell, Position},
    cursor::*,
    load::{Field, MineTextures, Textures},
    minefield::{systems::*, BlankField, GameOutcome, Minefield},
    state::ConditionalHelpersExt,
};
use bevy::prelude::*;
use iyes_loopless::{
    prelude::{AppLooplessStateExt, IntoConditionalSystem},
    state::NextState,
};

mod menu;

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
    field_templates: Res<Assets<BlankField>>,
    field_template: Res<Field>,
    mine_textures: Res<MineTextures>,
    textures: Res<Textures>,
) {
    // create minefield
    let field_template = field_templates.get(&field_template.field).unwrap();
    let minefield = Minefield::new_blank_shaped(&mut commands, &mine_textures, field_template);
    let minefield_entity = commands.spawn(()).insert(minefield).id();

    // get center of minefield
    #[allow(clippy::or_fun_call)]
    let init_position = field_template.center().unwrap_or(Position::new(0, 0));

    // create cursor
    commands
        .spawn(SpriteBundle {
            texture: textures.cursor.clone(),
            transform: Transform {
                translation: init_position.absolute(32.0, 32.0).extend(3.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cursor::new(CursorPosition(init_position, minefield_entity)));
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
            .add_system(
                // move_cursor
                pointer_cursor
                    .into_conditional()
                    .run_in_states([PreGame, Game]),
            )
            // .add_system(
            //     pointer_cursor.run_not_in_state(Inactive)
            // )
            .add_system(track_cursor.run_not_in_state(Inactive))
            .add_system(translate_cursor.run_not_in_state(Inactive))
            .add_system(init_check_cell.run_in_state(PreGame))
            .add_system(check_cell.run_in_state(Game));
    }
}
