use crate::{
    minefield::Minefield,
    server_v2::{
        app::player_connector_pair,
        double_channel::DoubleChannel,
        game::{GamemodeInitializer, SessionObjects},
    },
};

use super::components::ServerTile;

pub struct IAreaAttack;

impl GamemodeInitializer for IAreaAttack {
    fn create(&self, _: Vec<u8>) -> SessionObjects {
        // area attack has no params for now

        let (host_channel, own_channel) = DoubleChannel::<Vec<u8>>::double();
        let (connector_front, own_connector) = player_connector_pair();

        let main_task = tokio::spawn(async move {});

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

struct AreaAttack {
    field: Minefield<ServerTile>,
}
