//! An uncontrollable cursor which only serves the purpose of following the positions of peers.
//! Unlike a normal cursor, it can't be controlled by the player and doesn't send reveal events.

use bevy::{prelude::*, utils::HashMap};

use crate::common::Position;

#[derive(Component)]
pub struct PuppetCursor(pub Color);

#[derive(Bundle)]
pub struct PuppetCursorBundle {
    pub cursor: PuppetCursor,
    pub position: Position,
    pub sprite_bundle: SpriteBundle,
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct PuppetTable(HashMap<Entity, Entity>);

pub fn update_puppet_colors(
    mut q: Query<(&mut Sprite, &PuppetCursor), Or<(Added<PuppetCursor>, Changed<PuppetCursor>)>>,
) {
    for (mut sprite, &PuppetCursor(color_src)) in q.iter_mut() {
        sprite.color = color_src;
    }
}
