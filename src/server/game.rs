use bevy::prelude::*;

#[derive(Component)]
struct AreaAttack;

#[derive(Component, Deref, DerefMut)]
struct Players(Vec<Entity>);

#[derive(Bundle)]
struct GameBundle {
    players: Players,
}


#[derive(Bundle)]
struct AreaAttackBundle {
    marker: AreaAttack,
}
