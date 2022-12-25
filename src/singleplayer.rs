use crate::{
    common::InitCheckCell,
    cursor::*,
    load::{Field, Textures},
};
use crate::{
    main_menu::Menu,
    minefield::{
        specific::{MineCell, TILE_SIZE},
        systems::*,
        FieldShape, GameOutcome, Minefield,
    },
};
use bevy::prelude::*;
use iyes_loopless::{
    prelude::{AppLooplessStateExt, ConditionSet, IntoConditionalSystem},
    state::NextState,
};

mod menu;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Singleplayer {
    Inactive,
    PreGame,
    Game,
    Reset, // reserved for menu activation
    GameFailed,
    GameSuccess,
}

fn advance_to_end(mut commands: Commands, mut game_outcome: EventReader<GameOutcome>) {
    if let Some(outcome) = game_outcome.iter().next() {
        match outcome {
            GameOutcome::Failed => commands.insert_resource(NextState(Singleplayer::GameFailed)),
            GameOutcome::Succeeded => {
                commands.insert_resource(NextState(Singleplayer::GameSuccess))
            }
        }
    }
}

fn advance_to_game(mut commands: Commands, init_move: EventReader<InitCheckCell>) {
    if !init_move.is_empty() {
        commands.insert_resource(NextState(Singleplayer::Game))
    }
}

fn create_entities(
    mut commands: Commands,
    field_templates: Res<Assets<FieldShape>>,
    template_handles: Res<Field>,
    textures: Res<Textures>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    // create minefield
    let field_template = field_templates
        .get(template_handles.take_one(&mut rand::thread_rng()))
        .unwrap();
    let minefield = Minefield::new_shaped(
        |&pos| commands.spawn(MineCell::new_empty(pos, &textures)).id(),
        field_template,
    );

    // get starting position
    let init_position = field_template
        .center()
        .unwrap_or_else(|| minefield.iter_positions().next().unwrap());

    let minefield_entity = commands.spawn(()).insert(minefield).id();

    // move camera to cursor
    camera.single_mut().translation = init_position.absolute(TILE_SIZE, TILE_SIZE).extend(100.0);

    // create cursor
    commands.spawn(CursorBundle {
        cursor: Cursor::new(Color::YELLOW_GREEN, minefield_entity),
        position: init_position,
        texture: SpriteBundle {
            texture: textures.cursor.clone(),
            transform: Transform {
                translation: init_position.absolute(TILE_SIZE, TILE_SIZE).extend(3.0),
                ..default()
            },
            ..default()
        },
    });
}

pub struct SingleplayerMode;

impl Plugin for SingleplayerMode {
    fn build(&self, app: &mut App) {
        use Singleplayer::*;
        app
            // state
            .add_loopless_state(Inactive)
            // menu after game complete
            .add_plugin(menu::MenuPlugin)
            // state change startup and cleanup
            .add_exit_system(Inactive, create_entities)
            .add_system(generate_minefield.run_in_state(PreGame))
            .add_enter_system(PreGame, wipe_minefields)
            .add_system(advance_to_game.run_in_state(PreGame))
            .add_system(advance_to_end.run_in_state(Game))
            .add_enter_system(GameFailed, display_mines)
            // logic for leaving game
            .add_enter_system(Inactive, destroy_cursors)
            .add_enter_system(Inactive, destroy_minefields)
            // in-game logic
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(Menu::Pause)
                    .with_system(flag_cell.run_in_state(Game))
                    .with_system(reveal_cell.run_in_state(Game))
                    .with_system(init_check_cell.run_in_state(PreGame))
                    .with_system(check_cell.run_in_state(Game))
                    .into(),
            );
    }
}
