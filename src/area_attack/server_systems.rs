use bevy::prelude::*;

use crate::{
    load::Field,
    server::IngameEvent,
    singleplayer::minefield::{FieldShape, Minefield},
};

use super::{AreaAttackBundle, AREA_ATTACK_UUID};

#[derive(Component)]
struct Host;

pub fn create_game(
    mut commands: Commands,
    mut ev: EventReader<IngameEvent>,
    field_templates: Res<Assets<FieldShape>>,
    template_handles: Res<Field>,
) {
    for ev in ev.iter() {
        if let IngameEvent::Create {
            player, game, kind, ..
        } = ev
        {
            if **kind != AREA_ATTACK_UUID {
                continue;
            }

            let minefield = Minefield::new_shaped(
                |pos| commands.spawn(()).id(), // TODO Create cell entity types
                field_templates
                    .get(template_handles.take_one(&mut rand::thread_rng()))
                    .unwrap(),
            );

            commands
                .entity(*game)
                .insert(AreaAttackBundle::new(minefield));

            commands.entity(*player).insert(Host);
        }
    }
}
