use bevy::prelude::*;

pub trait QuicksweeperGame {
    type Bun: Bundle;

    fn name() -> &'static str;
    fn description() -> &'static str;
    fn initialize() -> Self::Bun;
}

#[derive(Component, Deref, DerefMut)]
pub struct Players(Vec<Entity>);

#[derive(Bundle)]
struct GameBundle {
    players: Players,
}
