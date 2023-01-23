//! An uncontrollable cursor which only serves the purpose of following the positions of peers.
//! Unlike a normal cursor, it can't be controlled by the player and doesn't send reveal events.

use bevy::prelude::*;

use crate::{common::Position, cursor::Cursor};

#[derive(Component, Deref, Copy, Clone)]
pub struct Puppet(pub Entity);

impl PartialEq<Entity> for Puppet {
    fn eq(&self, other: &Entity) -> bool {
        **self == *other
    }
}

#[derive(Bundle)]
pub struct PuppetCursorBundle {
    pub cursor: Cursor,
    pub position: Position,
    pub scene: SceneBundle,
    pub remote: Puppet,
}
