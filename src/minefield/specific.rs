use bevy::{gltf::Gltf, prelude::*};

use crate::{
    common::{Position, Vec2Ext},
    cursor::MainCursorMaterial,
    load::Textures,
};

/// Size of a single cell containing or not containing a mine. For now the display size of the mine
/// will be kept the same as the actual size of the sprite, but of course this will be subject to
/// change.
pub const TILE_SIZE: f32 = 2.0;

#[derive(Bundle)]
pub struct MineCell {
    sprite: SceneBundle,
    state: MineCellState,
    position: Position,
}

impl MineCell {
    pub fn new_empty(position: Position, textures: &Res<Textures>) -> Self {
        MineCell {
            sprite: SceneBundle {
                scene: textures.tile_empty.clone(),
                transform: Transform::from_translation(
                    position.absolute(TILE_SIZE, TILE_SIZE).extend_xz(0.0),
                ),
                ..default()
            },
            state: MineCellState::Empty,
            position,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Component)]
pub enum MineCellState {
    Empty,
    Mine,
    Revealed(u8),
    FlaggedEmpty,
    FlaggedMine,
}

impl MineCellState {
    pub fn is_flagged(&self) -> bool {
        matches!(
            self,
            MineCellState::FlaggedEmpty | MineCellState::FlaggedMine
        )
    }

    pub fn is_mine(&self) -> bool {
        matches!(self, MineCellState::Mine | MineCellState::FlaggedMine)
    }

    pub fn is_marked(&self) -> bool {
        !matches!(self, MineCellState::Mine | MineCellState::Empty)
    }
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
