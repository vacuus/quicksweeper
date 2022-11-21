use bevy::prelude::*;
use iyes_loopless::{
    prelude::{AppLooplessStateExt, IntoConditionalSystem},
    state::NextState,
};

use crate::{
    common::{InitCheckCell, Position},
    cursor::*,
    load::{Field, MineTextures, Textures},
    minefield::{systems::*, Minefield},
    minefield::{BlankField, GameOutcome},
    state::ConditionalHelpersExt,
};
pub struct MultiplayerMode;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum MultiplayerState {
    Inactive,
    GameFailed,
    GameSuccess,
    PreGame,
    Game,
}

fn create_entities(
    mut commands: Commands,
    field_templates: Res<Assets<BlankField>>,
    field_template: Res<Field>,
    mine_textures: Res<MineTextures>,
    textures: Res<Textures>,
) {
    let field_template = field_templates.get(&field_template.field).unwrap();
    let minefield = Minefield::new_blank_shaped(&mut commands, &mine_textures, field_template);
    let minefield_entity = commands.spawn(()).insert(minefield).id();

    #[allow(clippy::or_fun_call)]
    let init_position = field_template.center().unwrap_or(Position::new(0, 0));

    // player1
    commands
        .spawn(SpriteBundle {
            texture: textures.cursor.clone(),
            transform: Transform {
                translation: init_position.absolute(32.0, 32.0).extend(3.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cursor::new_with_keybindings(
            CursorPosition(init_position, minefield_entity),
            Bindings::default(),
        ));
    // player2
    commands
        .spawn(SpriteBundle {
            texture: textures.cursor.clone(),
            transform: Transform {
                translation: init_position.absolute(32.0, 32.0).extend(3.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cursor::new_with_keybindings(
            CursorPosition(init_position, minefield_entity),
            Bindings {
                left: KeyCode::Left,
                right: KeyCode::Right,
                up: KeyCode::Up,
                down: KeyCode::Down,
                flag: KeyCode::Slash,
                check: KeyCode::Return,
            },
        ));
}

fn advance_to_end(mut commands: Commands, mut game_outcome: EventReader<GameOutcome>) {
    if let Some(outcome) = game_outcome.iter().next() {
        match outcome {
            GameOutcome::Failed => {
                commands.insert_resource(NextState(MultiplayerState::GameFailed))
            }
            GameOutcome::Succeeded => {
                commands.insert_resource(NextState(MultiplayerState::GameSuccess))
            }
        }
    }
}

fn advance_to_game(mut commands: Commands, init_move: EventReader<InitCheckCell>) {
    if !init_move.is_empty() {
        println!("moving to ingame mode");
        commands.insert_resource(NextState(MultiplayerState::Game))
    }
}

impl Plugin for MultiplayerMode {
    fn build(&self, app: &mut App) {
        use MultiplayerState::*;
        app.add_loopless_state(MultiplayerState::Inactive)
            // state transitions
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
                move_cursor
                    .into_conditional()
                    .run_in_states([PreGame, Game]),
            )
            .add_system(translate_cursor.run_not_in_state(Inactive))
            .add_system(init_check_cell.run_in_state(PreGame))
            .add_system(check_cell.run_in_state(Game));
    }
}
