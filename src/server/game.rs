use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GameDescriptor {
    pub name: String,
    pub description: String,
}

pub trait QuicksweeperGame {
    type Bun: Bundle;

    fn descriptor(&self) -> GameDescriptor;
    fn initialize(&self) -> Self::Bun;
}

#[derive(Component, Deref, DerefMut)]
pub struct Players(Vec<Entity>);

#[derive(Bundle)]
struct GameBundle {
    players: Players,
}
