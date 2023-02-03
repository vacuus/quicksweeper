use rand::seq::SliceRandom;

use crate::{
    minefield::{FieldShape, Minefield},
    server_v2::{
        app::{player_connector_pair, PlayerReceiver},
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
        let (connector_front, player_receiver) = player_connector_pair();
        let field_shape = FIELDS.choose(&mut rand::thread_rng()).unwrap().clone();
        let field = Minefield::new_shaped(|_| ServerTile::Empty, &field_shape);

        let game = AreaAttack {
            field_shape,
            field,
            players: vec![host_listener],
            player_receiver,
        };

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
    field_shape: FieldShape,
    field: Minefield<ServerTile>,
    /// The first player in this list shall be considered the host
    players: Vec<DoubleChannel<Vec<u8>>>,
    player_receiver: PlayerReceiver,
}
