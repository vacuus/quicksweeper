use std::{collections::HashMap, sync::Arc};

use itertools::Itertools;
use tokio::{
    net::TcpListener,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex, RwLock,
    }, task::JoinHandle,
};
use tokio_tungstenite as tungsten;
use unique_id::sequence::SequenceGenerator;

use crate::server::{ActiveGame, GameMarker, Greeting};

use super::{connection::Connection, double_channel::DoubleChannel};

pub struct GameConnector {
    request: Sender<Greeting>,
    recv: Receiver<DoubleChannel<Vec<u8>>>,
}

pub struct GameHandle {
    kind: GameMarker,
    players: Arc<Vec<String>>,
    connect: Mutex<GameConnector>,
    task_handle: JoinHandle<()>,
}

#[derive(Clone, Default)]
pub struct GameList(Arc<RwLock<HashMap<u64, GameHandle>>>);

impl GameList {
    pub async fn list(&self) -> Vec<ActiveGame> {
        self.0
            .read()
            .await
            .iter()
            .map(|(&id, handle)| ActiveGame {
                marker: handle.kind,
                id,
                players: (*handle.players).clone(),
            })
            .collect_vec()
    }
}

pub struct App {
    games: GameList,
    generator: Arc<SequenceGenerator>,
    listener: TcpListener,
}

impl App {
    pub async fn new(address: String) -> Self {
        let listener = TcpListener::bind((address, 0)).await.unwrap();
        log::debug!("Server open at {}", listener.local_addr().unwrap());

        Self {
            games: Default::default(),
            generator: Arc::new(SequenceGenerator::default()),
            listener,
        }
    }

    pub async fn run(self) {
        while let Ok((sock, _addr)) = self.listener.accept().await {
            let games = self.games.clone();
            let generator = self.generator.clone();
            tokio::spawn(async {
                let sock = Connection(tungsten::accept_async(sock).await?);
                match sock.upgrade(games, generator).await {
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
