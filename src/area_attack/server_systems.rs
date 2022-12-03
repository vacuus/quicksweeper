use bevy::prelude::*;
use strum::IntoEnumIterator;

use crate::{
    load::Field,
    server::{Connection, ConnectionInfo, IngameEvent, MessageSocket, Access},
    singleplayer::minefield::FieldShape,
};

use super::{
    components::{AreaAttackBundle, PlayerBundle, PlayerColor},
    protocol::AreaAttackUpdate,
    AreaAttackServer, AREA_ATTACK_MARKER,
};

#[derive(Component)]
struct Host;

pub fn create_game(
    mut commands: Commands,
    mut ev: EventReader<IngameEvent>,
    field_templates: Res<Assets<FieldShape>>,
    template_handles: Res<Field>,
    mut update_readiness: Local<Vec<Entity>>,
    mut access: Query<&mut Access>,
) {

    for game in update_readiness.iter() {
        *access.get_mut(*game).unwrap() = Access::Open;
    }

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
            update_readiness.push(*game);
        }
    }
}

pub fn prepare_player(
    mut commands: Commands,
    mut ev: EventReader<IngameEvent>,
    games: Query<(&Children, &FieldShape), With<AreaAttackServer>>,
    players: Query<(&ConnectionInfo, &mut PlayerColor)>,
    mut connections: Query<&mut Connection>,
) {
    for ev in ev.iter() {
        println!("`prepare_player` received event {ev:?}");
        let IngameEvent::Join { player, game } = ev else { println!("Broken from guard 1"); continue; };
        let Ok((peers, shape)) = games.get(*game) else { println!("Broken from guard 2"); continue; };
        let Ok(mut this_connection) = connections.get_mut(*player) else {println!("Broken from guard 3"); continue; };

        // guard if there are too many players
        if peers.len() >= 4 {
            let _ = this_connection.send_message(AreaAttackUpdate::Full);
            commands.entity(*game).remove_children(&[*player]);
            continue;
        }

        let mut taken_colors = Vec::new();

        // send the selected board
        let _ = this_connection.send_message(AreaAttackUpdate::FieldShape(shape.clone()));

        // send list of players and player properties
        for &peer_id in peers {
            let (ConnectionInfo { username }, &color) = players.get(peer_id).unwrap();
            taken_colors.push(color);
            let _ = this_connection.send_message(AreaAttackUpdate::PlayerModified {
                id: peer_id,
                username: username.clone(),
                color,
            });
        }

        let assigned_color = PlayerColor::iter()
            .find(|co| !taken_colors.contains(co))
            .unwrap();

        // initialize some player properties
        commands.entity(*player).insert(PlayerBundle {
            color: assigned_color,
        });
        let _ = this_connection.send_message(AreaAttackUpdate::SelfModified {
            color: assigned_color,
        });
    }
}
