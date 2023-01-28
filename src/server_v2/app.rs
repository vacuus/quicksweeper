use std::sync::Arc;

use tokio::{
    net::TcpListener,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
};

use crate::server::GameMarker;

use super::double_channel::DoubleChannel;

pub struct GameConnector {
    request: Sender<bool>,
    recv: Receiver<DoubleChannel<Vec<u8>>>,
}

pub struct GameHandle {
    kind: GameMarker,
    connect: Mutex<GameConnector>,
}

#[derive(Clone, Default)]
pub struct GameList(Arc<Vec<GameHandle>>);

pub struct App {
    games: GameList,
    listener: TcpListener,
}

impl App {
    pub async fn new(address: String) -> Self {
        Self {
            games: Default::default(),
            listener: TcpListener::bind(address).await.unwrap(),
        }
    }
}
