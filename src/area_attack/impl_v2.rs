
use rand::seq::SliceRandom;
use tokio::pin;

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

pub struct IAreaAttack;

impl GamemodeInitializer for IAreaAttack {
    fn create(&self, _: Vec<u8>) -> SessionObjects {
        // area attack has no params for now

        let (host_channel, host_listener) = DoubleChannel::<Vec<u8>>::double();
        let (connector_front, mut player_receiver) = player_connector_pair();
        let field_shape = FIELDS.choose(&mut rand::thread_rng()).unwrap().clone();
        let field = Minefield::new_shaped(|_| ServerTile::Empty, &field_shape);

        pin!(host_listener);
        let mut players = vec![host_listener];

        let main_task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(player) = player_receiver.recv() => {

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
