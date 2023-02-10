use std::collections::{HashMap, HashSet};

use futures_util::{stream::FuturesUnordered, StreamExt};
use rand::seq::SliceRandom;
use tokio::sync::mpsc::{error::SendError, UnboundedSender};

use crate::{
    minefield::Minefield,
    server::Greeting,
    server_v2::{
        app::player_connector_pair,
        double_channel::DoubleChannel,
        game::{GamemodeInitializer, SessionObjects},
        FIELDS,
    },
};

use super::{components::{ServerTile, PlayerColor}, protocol::AreaAttackRequest};

#[derive(Default)]
struct PlayerSet {
    map: HashMap<Greeting, SendOnlyPlayer>,
    taken_colors: HashSet<PlayerColor>,
}

impl PlayerSet {
    fn register(&mut self, info: Greeting, chan: DoubleChannel<Vec<u8>>) -> Player {
        // assign a color to the player
        // notify other players of this player
        // register player into map
        // return double-channel player
        todo!()
    }
}

struct Player {
    is_host: bool,
    info: Greeting,
    connector: DoubleChannel<Vec<u8>>,
}

impl Player {
    fn send(&mut self, message: Vec<u8>) -> Result<(), SendError<Vec<u8>>> {
        self.connector.send(message)
    }

    async fn recv_owned(mut self) -> (Option<Vec<u8>>, Self) {
        (self.connector.recv().await, self)
    }

    fn sender(&self) -> SendOnlyPlayer {
        SendOnlyPlayer {
            is_host: self.is_host,
            info: self.info.clone(),
            connector: self.connector.sender(),
        }
    }
}

struct SendOnlyPlayer {
    is_host: bool,
    info: Greeting,
    connector: UnboundedSender<Vec<u8>>,
}

pub struct IAreaAttack;

impl GamemodeInitializer for IAreaAttack {
    fn create(&self, _: Vec<u8>, info: Greeting) -> SessionObjects {
        // area attack has no params for now

        let (host_channel, host_listener) = DoubleChannel::<Vec<u8>>::double();
        let (connector_front, mut player_receiver) = player_connector_pair();
        let field_shape = FIELDS.choose(&mut rand::thread_rng()).unwrap().clone();
        let field = Minefield::new_shaped(|_| ServerTile::Empty, &field_shape);

        let mut player_set = PlayerSet::default();

        let main_task = tokio::spawn(async move {
            let host = player_set.register(info, host_listener);
            let mut task_queue = FuturesUnordered::new();
            task_queue.push(host.recv_owned());

            loop {
                tokio::select! {
                    Some(player) = player_receiver.recv() => {
                        let player_listener = player_receiver.respond().await.unwrap();
                        player_set.register(player, player_listener);
                    }
                    Some((player_msg, mut player)) = task_queue.next() => {
                        if let Some(msg) = player_msg {
                            if let Ok(message) = rmp_serde::from_slice(&msg) {
                                handle_message(message, &mut player_set, &mut player);
                            }
                            task_queue.push(player.recv_owned());
                        }
                    }
                }
            }
        });

        SessionObjects {
            host_channel,
            connector: connector_front,
            main_task,
        }
    }
}

fn handle_message(
    msg: AreaAttackRequest,
    senders: &mut PlayerSet,
    player: &mut Player,
) {
    use AreaAttackRequest::*;
    match msg {
        StartGame => (),
        Reveal(_) => (),
        Position(_) => (),
        Color(_) => (),
    }
}

impl IAreaAttack {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}
