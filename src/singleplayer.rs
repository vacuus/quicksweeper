use crate::{
    common::{InitCheckCell, Vec2Ext},
    cursor::*,
    load::{Field, Textures}, minefield::specific::MineCellState,
};
use crate::{
    main_menu::Menu,
    minefield::{
        specific::{MineCell, TILE_SIZE},
        systems::*,
        FieldShape, GameOutcome, Minefield,
    },
};
use bevy::{prelude::*, gltf::Gltf};
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
    mut camera: Query<&mut Transform, With<Camera>>,
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
    let mut camera = camera.single_mut();
    camera.translation.x = TILE_SIZE * init_position.x as f32;
    camera.translation.z = TILE_SIZE * init_position.y as f32;

    // create cursor
    commands.spawn(CursorBundle {
        cursor: Cursor::new(Color::YELLOW_GREEN, minefield_entity),
        position: init_position,
        texture: SceneBundle {
            scene: textures.cursor.clone(),
            transform: Transform {
                translation: init_position.absolute(TILE_SIZE, TILE_SIZE).extend_xz(1.0),
                ..default()
            },
            ..default()
        }// ,
    });
}

#[derive(Component, Deref)]
pub struct NeedsMaterial(Handle<StandardMaterial>); // TODO fill this in with material handle instead of color

pub fn update_tiles(
    mut commands: Commands,
    mut changed_cells: Query<
        (Entity, &mut Handle<Scene>, &MineCellState),
        Or<(Added<MineCellState>, Changed<MineCellState>)>,
    >,
    player_material: Res<MainCursorMaterial>,
    // cursor: Query<&Cursor>,
    textures: Res<Textures>,
    gltf: Res<Assets<Gltf>>,
) {
    // let color = cursor.get_single().map(|c| c.color).unwrap_or(Color::WHITE);
    changed_cells.for_each_mut(|(tile_id, mut scene, state)| {
        *scene = match state {
            MineCellState::Empty | MineCellState::Mine => textures.tile_empty.clone(),
            MineCellState::FlaggedMine | MineCellState::FlaggedEmpty => {
                commands
                    .entity(tile_id)
                    .insert((NeedsMaterial(player_material.clone()),));
                textures.tile_flagged.clone()
            }
            &MineCellState::Revealed(x) => {
                commands
                    .entity(tile_id)
                    .insert((NeedsMaterial(player_material.clone()),));
                gltf.get(&textures.mines_3d).unwrap().named_scenes[&format!("f.tile_filled.{x}")]
                    .clone()
            }
        };
    })
}

pub fn update_tile_colors(
    mut commands: Commands,
    mut new_meshes: Query<(Entity, &mut Handle<StandardMaterial>, &Name), Added<Handle<Mesh>>>,
    color_requested: Query<&NeedsMaterial>,
    parents: Query<&Parent>,
) {
    let parent_of = |mut id: Entity, recursions: usize| {
        for _ in 0..recursions {
            id = **parents.get(id).unwrap()
        }
        id
    };

    for (tile_id, mut material, name) in &mut new_meshes {
        if name.contains("Text") || &**name == ("tile_empty"){
            continue;
        }
        let root = parent_of(tile_id, 3);
        if let Ok(color) = color_requested.get(root) {
            // TODO assign color here
            *material = (*color).clone();
        }
        commands.entity(root).remove::<NeedsMaterial>();
    }
}

pub struct SingleplayerMode;

impl Plugin for SingleplayerMode {
    fn build(&self, app: &mut App) {
        use Singleplayer::*;
        app
            // tile draw
            .add_system(update_tile_colors)
            .add_system(update_tiles.run_not_in_state(Menu::Loading))
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
