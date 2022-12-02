use bevy::prelude::*;

use crate::{
    load::Field,
    server::{Connection, ConnectionInfo, IngameEvent, MessageSocket},
    singleplayer::minefield::FieldShape,
};

use super::{
    protocol::{AreaAttackUpdate, PlayerColor},
    AreaAttackBundle, AreaAttackServer, AREA_ATTACK_MARKER,
};

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
            if *kind != AREA_ATTACK_MARKER {
                continue;
            }

            let template = template_handles.take_one(&mut rand::thread_rng()).clone();

            let bundle = AreaAttackBundle::new(&mut commands, template, &field_templates);
            commands.entity(*game).insert(bundle);

            commands.entity(*player).insert(Host);
        }
    }
}

pub fn prepare_player(
    mut commands: Commands,
    mut ev: EventReader<IngameEvent>,
    games: Query<(&Children, &FieldShape), With<AreaAttackServer>>,
    players: Query<(&ConnectionInfo, &PlayerColor)>,
    mut connections: Query<&mut Connection>,
) {
    for ev in ev.iter() {
        let IngameEvent::Join { player, game } = ev else { continue; };
        let Ok((peers, shape)) = games.get(*game) else { continue; };
        let Ok(mut this_connection) = connections.get_mut(*player) else { continue; };

        // send the selected board
        let _ = this_connection.send_message(AreaAttackUpdate::FieldShape(shape.clone()));

        // send list of players and player properties
        for &peer_id in peers {
            let (ConnectionInfo { username }, &color) = players.get(peer_id).unwrap();
            let _ = this_connection.send_message(AreaAttackUpdate::PlayerModified {
                id: peer_id,
                username: username.clone(),
                color,
            });
        }

        // initialize some player properties
        unimplemented!("initialize the player entity itself") // TODO implement
    }
}
