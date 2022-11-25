use bevy::prelude::*;

pub struct GameDescriptor {
    pub name: &'static str,
    pub description: &'static str
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
