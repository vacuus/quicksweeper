use bevy::prelude::*;

use crate::{singleplayer::minefield::Minefield, server::{QuicksweeperGame, GameDescriptor}};

#[derive(Component)]
struct AreaAttack;

#[derive(Bundle)]
struct AreaAttackBundle {
    marker: AreaAttack,
    minefield: Minefield,
}

impl QuicksweeperGame for AreaAttack {
    type Bun = AreaAttackBundle;

    fn initialize(&self) -> Self::Bun {
        AreaAttackBundle {
            marker: AreaAttack,
            minefield: Minefield::new_shaped(|_pos| todo!(), todo!()),
        }
    }

    fn descriptor(&self) -> GameDescriptor {
        GameDescriptor {
            name: "Area Attack".to_string(),
            description: "A race to claim the entire board for yourself".to_string(),
        }
    }
}