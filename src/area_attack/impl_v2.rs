use futures_util::{stream::FuturesUnordered, StreamExt};
use rand::seq::SliceRandom;
use tap::Tap;
use tokio::sync::mpsc::error::SendError;

use crate::{
    minefield::Minefield,
    server_v2::{
        app::player_connector_pair,
        double_channel::DoubleChannel,
        game::{GamemodeInitializer, SessionObjects},
        FIELDS,
    },
};

use super::components::ServerTile;

struct Player {
    is_host: bool,
    connector: DoubleChannel<Vec<u8>>,
}

impl From<DoubleChannel<Vec<u8>>> for Player {
    fn from(connector: DoubleChannel<Vec<u8>>) -> Self {
        Self {
            is_host: false,
            connector,
        }
    }
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
    fn create(&self, _: Vec<u8>) -> SessionObjects {
        // area attack has no params for now

        let (host_channel, host_listener) = DoubleChannel::<Vec<u8>>::double();
        let (connector_front, mut player_receiver) = player_connector_pair();
        let field_shape = FIELDS.choose(&mut rand::thread_rng()).unwrap().clone();
        let field = Minefield::new_shaped(|_| ServerTile::Empty, &field_shape);

        let main_task = tokio::spawn(async move {
            let host = Player::from(host_listener).tap_mut(|p| p.is_host = true);
            let mut task_queue = FuturesUnordered::new();
            task_queue.push(host.recv_owned());

            loop {
                tokio::select! {
                    Some(player) = player_receiver.recv() => {

                    }
                    Some((player_msg, player)) = task_queue.next() => {
                        task_queue.push(player.recv_owned())
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

impl IAreaAttack {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}
