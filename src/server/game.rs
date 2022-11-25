//! ## How to create a quicksweeper game
//!
//! A quicksweeper gamemode is an entity that contains a [GameBundle] and "responds" to
//! [ServerEvent]s. It should contain a system which generates a new game when a
//! [ServerEvent::Create] event is issued to it, and should receive data when a [ServerEvent::Data]
//! event is issued to it. The event contains a Uuid which signals to your system whether or not
//! your gamemode is being targeted.
//!
//!

use std::collections::HashMap;

use bevy::{prelude::*, utils::Uuid};

use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Debug)]
pub struct GameDescriptor {
    pub name: String,
    pub description: String,
}

#[derive(Component, Deref, DerefMut, Serialize, Deserialize, Debug)]
pub struct GameMarker(pub Uuid);

#[derive(Component, Deref, DerefMut, Default)]
pub struct Players(pub Vec<Entity>);

#[derive(Bundle)]
pub struct GameBundle {
    pub marker: GameMarker,
    pub descriptor: GameDescriptor,
    pub players: Players,
}