use bevy::{hierarchy::HierarchyEvent, prelude::*};
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
    load::Field,
    server::{
        Access, Connection, ConnectionInfo, ConnectionSwitch, GameMarker, IngameEvent,
        MessageSocket,
    },
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
    new_games: Query<(Entity, &GameMarker, &Children), Added<GameMarker>>,
    field_templates: Res<Assets<FieldShape>>,
    template_handles: Res<Field>,
) {
    for (game, kind, children) in new_games.iter() {
        if *kind != AREA_ATTACK_MARKER {
            continue;
        }

        // initialize game variables
        let template = template_handles.take_one(&mut rand::thread_rng()).clone();
        let bundle = AreaAttackBundle::new(&mut commands, template, &field_templates);
        commands.entity(game).insert(bundle);

        // there should only be one player right now, mark it as host
        let &player = children.iter().exactly_one().unwrap();
        commands.entity(player).insert(Host);
    }
}

pub fn unmark_init_access(mut access: Query<&mut Access, Added<AreaAttackServer>>) {
    access.for_each_mut(|mut access| {
        println!("Opening access to area attack");
        *access = Access::Open;
    })
}

pub fn prepare_player(
    mut commands: Commands,
    mut ev: EventReader<ConnectionSwitch>,
    games: Query<(&Children, &FieldShape), With<AreaAttackServer>>,
    players: Query<(&ConnectionInfo, &mut PlayerColor)>,
    mut connections: Query<&mut Connection>,
) {
    for ev in ev.iter() {
        println!("`prepare_player` received event {ev:?}");
        let ConnectionSwitch(HierarchyEvent::ChildAdded { child: player, parent: game }) = ev else {
            // TODO add ChildMoved variant as well
            continue;
        };
        let Ok((peers, shape)) = games.get(*game) else { continue; };
        let Ok(mut this_connection) = connections.get_mut(*player) else {continue; };

        // guard if there are too many players
        if peers.len() > 4 {
            let _ = this_connection.send_message(AreaAttackUpdate::Full);
            commands.entity(*game).remove_children(&[*player]);
            continue;
        }

        let mut taken_colors = Vec::new();

        // send the selected board
        let _ = this_connection.send_message(AreaAttackUpdate::FieldShape(shape.clone()));

        // send list of players and player properties
        for &peer_id in peers {
            if peer_id == *player {
                continue;
            }
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
