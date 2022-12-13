use bevy::{hierarchy::HierarchyEvent, prelude::*};
use itertools::Itertools;
use strum::IntoEnumIterator;

use crate::{
    common::Position,
    load::Field,
    server::{
        Access, Connection, ConnectionInfo, ConnectionSwitch, GameMarker, IngameEvent, LocalEvent,
        MessageSocket,
    },
    singleplayer::minefield::FieldShape,
};

use super::{
    components::{AreaAttackBundle, PlayerBundle, PlayerColor},
    protocol::{AreaAttackRequest, AreaAttackUpdate},
    states::AreaAttackState,
    AreaAttackServer, AREA_ATTACK_MARKER,
};

#[derive(Component)]
pub struct Host;

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

// transcribes incoming messages to area attack events
pub fn net_events(
    mut network: EventReader<IngameEvent>,
    mut local: EventWriter<LocalEvent<AreaAttackRequest>>,
    games: Query<Entity, With<AreaAttackServer>>,
) {
    eprintln!("{:?}", games.iter().collect_vec());
    for ige @ IngameEvent { game, .. } in network.iter() {
        println!("Got event, no filter");
        if games.contains(*game) {
            println!("Filter 1 passed");
            if let Ok(msg) = ige.transcribe() {
                println!("Area attack message received: {msg:?}");
                local.send(msg);
            }
        }
    }
}

pub fn selection_transition(
    mut ev: EventReader<LocalEvent<AreaAttackRequest>>,
    mut games: Query<&mut AreaAttackState>,
    maybe_host: Query<(), With<Host>>,
) {
    for ev in ev.iter() {
        if !matches!(ev.data, AreaAttackRequest::StartGame) {
            continue;
        }
        if let Ok(mut state) = games.get_mut(ev.game) {
            if maybe_host.get(ev.player).is_ok() && matches!(*state, AreaAttackState::Selecting) {
                *state = AreaAttackState::Stage1;
                println!("Selection transition begun!")
            }
        }
    }
}

pub fn prepare_player(
    mut commands: Commands,
    mut ev: EventReader<ConnectionSwitch>,
    mut games: Query<(&Children, &FieldShape, &mut Access), With<AreaAttackServer>>,
    players: Query<(&ConnectionInfo, &mut PlayerColor, &Position)>,
    mut connections: Query<&mut Connection>,
) {
    for ev in ev.iter() {
        println!("`prepare_player` received event {ev:?}");
        let ConnectionSwitch(HierarchyEvent::ChildAdded { child: player, parent: game }) = ev else {
            // TODO add ChildMoved variant as well
            continue;
        };
        let Ok((peers, shape, mut access)) = games.get_mut(*game) else { continue; };
        let Ok(mut this_connection) = connections.get_mut(*player) else {continue; };

        // guard if there are too many players
        if peers.len() >= 4 {
            *access = Access::Full // TODO: Reset when connection drops
        }

        let mut taken_colors = Vec::new();

        // send the selected board
        let _ = this_connection.send_message(AreaAttackUpdate::FieldShape(shape.clone()));

        // send list of players and player properties
        for &peer_id in peers {
            if peer_id == *player {
                continue;
            }
            let (ConnectionInfo { username }, &color, &position) = players.get(peer_id).unwrap();
            taken_colors.push(color);
            let _ = this_connection.send_message(AreaAttackUpdate::PlayerProperties {
                id: peer_id,
                username: username.clone(),
                color,
                position,
            });
        }

        let assigned_color = PlayerColor::iter()
            .find(|co| !taken_colors.contains(co))
            .unwrap();

        // initialize some player properties
        let position = shape.center().unwrap_or(Position { x: 0, y: 0 });
        commands.entity(*player).insert(PlayerBundle {
            color: assigned_color,
            position,
        });
        let _ = this_connection.send_message(AreaAttackUpdate::SelfChange {
            color: assigned_color,
        });
    }
}
