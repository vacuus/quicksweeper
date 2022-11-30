use bevy::prelude::*;

use crate::{server::IngameEvent, singleplayer::minefield::Minefield};

use super::AreaAttackBundle;

pub fn create_game(mut commands: Commands, mut ev: EventReader<IngameEvent>) {
    for ev in ev.iter() {
        if let IngameEvent::Create {
            player,
            game,
            kind,
            data,
        } = ev
        {
            // commands
            //     .entity(*game)
            //     .insert(AreaAttackBundle::new(Minefield::new_shaped(
            //         |pos| {todo!()},
            //         todo!(),
            //     )));
        }
    }
}
