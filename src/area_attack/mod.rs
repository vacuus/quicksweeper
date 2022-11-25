use bevy::prelude::*;

use crate::{singleplayer::minefield::Minefield, server::QuicksweeperGame};

#[derive(Component)]
struct AreaAttack;

#[derive(Bundle)]
struct AreaAttackBundle {
    marker: AreaAttack,
    minefield: Minefield,
}

impl QuicksweeperGame for AreaAttack {
    type Bun = AreaAttackBundle;

    fn name() -> &'static str {
        "Area Attack"
    }

    fn description() -> &'static str {
        "A race to claim the entire board for yourself"
    }

    fn initialize() -> Self::Bun {
        AreaAttackBundle {
            marker: AreaAttack,
            minefield: Minefield::new_shaped(|_pos| todo!(), todo!()),
        }
    }
}