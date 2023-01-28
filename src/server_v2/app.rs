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

pub struct GameInfo {
    pub kind: GameMarker,
}

pub struct GameHandle {
    info: GameInfo,
    connect: Mutex<GameConnector>,
}

pub struct App {
    games: Vec<GameHandle>,
    listener: TcpListener,
}

impl App {
    pub async fn new(address: String) -> Arc<Self> {
        Arc::new(Self {
            games: Default::default(),
            listener: TcpListener::bind(address).await.unwrap(),
        })
    }
}
