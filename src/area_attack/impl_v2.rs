use std::collections::HashMap;

use futures_util::{stream::FuturesUnordered, Future, StreamExt};
use rand::seq::SliceRandom;
use tap::Tap;
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

use super::{
    components::ServerTile,
    protocol::{AreaAttackRequest, AreaAttackUpdate},
};

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
}

pub struct IAreaAttack;

impl GamemodeInitializer for IAreaAttack {
    fn create(&self, _: Vec<u8>, info: Greeting) -> SessionObjects {
        // area attack has no params for now

        let (host_channel, host_listener) = DoubleChannel::<Vec<u8>>::double();
        let (connector_front, mut player_receiver) = player_connector_pair();
        let field_shape = FIELDS.choose(&mut rand::thread_rng()).unwrap().clone();
        let field = Minefield::new_shaped(|_| ServerTile::Empty, &field_shape);

        let mut senders = HashMap::new();

        let main_task = tokio::spawn(async move {
            senders.insert(info.clone(), host_listener.sender());
            let host = Player {
                is_host: true,
                info,
                connector: host_listener,
            };
            let mut task_queue = FuturesUnordered::new();
            task_queue.push(host.recv_owned());

            loop {
                tokio::select! {
                    Some(player) = player_receiver.recv() => {

                    }
                    Some((player_msg, mut player)) = task_queue.next() => {
                        if let Some(msg) = player_msg {
                            if let Ok(message) = rmp_serde::from_slice(&msg) {
                                handle_message(message, &mut senders, &mut player);
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
    senders: &mut HashMap<Greeting, UnboundedSender<Vec<u8>>>,
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
