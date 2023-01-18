//! An uncontrollable cursor which only serves the purpose of following the positions of peers.
//! Unlike a normal cursor, it can't be controlled by the player and doesn't send reveal events.

use bevy::prelude::*;

use crate::{common::Position, cursor::Cursor};

#[derive(Component)]
pub struct PuppetCursor(pub Color);

#[derive(Component, Deref, Copy, Clone)]
pub struct Remote(pub Entity);

impl PartialEq<Entity> for Remote {
    fn eq(&self, other: &Entity) -> bool {
        **self == *other
    }
}

#[derive(Bundle)]
pub struct PuppetCursorBundle {
    pub cursor: PuppetCursor,
    pub position: Position,
    pub sprite_bundle: SceneBundle,
    pub remote: Remote,
}

pub fn update_cursor_colors(
    mut q_set: ParamSet<(
        Query<(&mut Sprite, &PuppetCursor), Or<(Added<PuppetCursor>, Changed<PuppetCursor>)>>,
        Query<(&mut Sprite, &Cursor), Or<(Added<Cursor>, Changed<Cursor>)>>,
    )>,
) {
    for (mut sprite, &PuppetCursor(color_src)) in q_set.p0().iter_mut() {
        sprite.color = color_src;
    }

    for (
        mut sprite,
        &Cursor {
            color: color_src, ..
        },
    ) in q_set.p1().iter_mut()
    {
        sprite.color = color_src;
    }
}
