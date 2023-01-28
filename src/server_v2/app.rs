use std::sync::Arc;

use tokio::{
    net::TcpListener,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex, RwLock,
    },
};
use tokio_tungstenite as tungsten;

use crate::server::GameMarker;

use super::{connection::Connection, double_channel::DoubleChannel};

pub struct GameConnector {
    request: Sender<bool>,
    recv: Receiver<DoubleChannel<Vec<u8>>>,
}

pub struct GameHandle {
    kind: GameMarker,
    connect: Mutex<GameConnector>,
}

#[derive(Clone, Default)]
pub struct GameList(Arc<RwLock<Vec<GameHandle>>>);

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

    pub async fn run(self) {
        while let Ok((sock, _addr)) = self.listener.accept().await {
            let games = self.games.clone();
            tokio::spawn(async {
                let sock = Connection(tungsten::accept_async(sock).await?);
                match sock.upgrade(games).await {
                    Ok(mut player) => {
                        player.enter().await;
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            });
        }
    }
}
