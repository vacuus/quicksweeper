//! ## How to create a quicksweeper game
//!
//! A quicksweeper gamemode is an entity that contains a [GameBundle] and "responds" to
//! [ServerEvent]s. It should contain a system which generates a new game when a
//! [ServerEvent::Create] event is issued to it, and should receive data when a [ServerEvent::Data]
//! event is issued to it. The event contains a Uuid which signals to your system whether or not
//! your gamemode is being targeted.
//!
//!

use bevy::{prelude::*, utils::Uuid};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GameDescriptor {
    pub name: String,
    pub description: String,
}

#[derive(Component, Deref, DerefMut)]
pub struct GameMarker(pub Uuid);

#[derive(Component, Deref, DerefMut, Default)]
pub struct Players(pub Vec<Entity>);

#[derive(Bundle)]
pub struct GameBundle {
    pub marker: GameMarker,
    pub players: Players,
}

pub enum ServerEvent {
    Data {
        player: Entity,
        game: Entity,
        marker: GameMarker,
        data: Vec<u8>,
    },
    Create {
        player: Entity,
        marker: GameMarker,
        params: Vec<u8>,
    }
}
