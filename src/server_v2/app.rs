use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use itertools::Itertools;
use tokio::{
    net::TcpListener,
    sync::{
        mpsc::{Receiver, Sender},
        Mutex, RwLock,
    },
    task::JoinHandle,
};
use tokio_tungstenite as tungsten;
use unique_id::{sequence::SequenceGenerator, Generator};

use crate::{
    registry::REGISTRY,
    server::{ActiveGame, GameDescriptor, GameMarker, Greeting},
};

use super::{connection::Connection, double_channel::DoubleChannel, game::GameComponents};

pub struct GameConnector {
    num_connections: Arc<AtomicU64>,
    request: Sender<Greeting>,
    recv: Receiver<DoubleChannel<Vec<u8>>>,
}

impl GameConnector {
    pub async fn connect(&mut self, player: Greeting) -> Option<DoubleChannel<Vec<u8>>> {
        if self.request.send(player).await.is_ok() {
            if let Some(result) = self.recv.recv().await {
                self.num_connections.fetch_add(1, Ordering::SeqCst);
                Some(result)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct GameHandle {
    kind: GameMarker,
    players: Arc<Vec<String>>,
    connect: Mutex<GameConnector>,
    task_handle: JoinHandle<()>,
}

#[derive(Clone, Default)]
pub struct GameStore {
    store: Arc<RwLock<HashMap<u64, GameHandle>>>,
    generator: Arc<SequenceGenerator>,
}

impl GameStore {
    pub async fn list(&self) -> Vec<ActiveGame> {
        self.store
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

    pub async fn join(&self, game_id: &u64, player: Greeting) -> Option<DoubleChannel<Vec<u8>>> {
        if let Some(game) = self.store.read().await.get(game_id) {
            if let Some(chan) = game.connect.lock().await.connect(player).await {
                Some(chan)
            } else {
                if let Some(GameHandle { task_handle, .. }) =
                    self.store.write().await.remove(game_id)
                {
                    task_handle.abort();
                }
                None
            }
        } else {
            None
        }
    }

    pub async fn create_new(&self, game: &GameMarker, args: Vec<u8>) -> Option<DoubleChannel<Vec<u8>>> {
        if let Some(GameDescriptor { initializer, .. }) = REGISTRY.get(game) {
            let GameComponents {
                host_channel,
                connector,
                main_task,
            } = initializer.create(args);
            let key = self.generator.next_id() as u64; //TODO generator reset? Or some way to prevent crashes

            self.store.write().await.insert(
                key,
                GameHandle {
                    kind: *game,
                    players: Arc::new(Vec::new()),
                    connect: Mutex::new(connector),
                    task_handle: main_task,
                },
            );
            Some(host_channel)
        } else {
            None
        }
    }
}

pub struct App {
    games: GameStore,
    listener: TcpListener,
}

impl App {
    pub async fn new(address: String) -> Self {
        let listener = TcpListener::bind((address, 0)).await.unwrap();
        log::debug!("Server open at {}", listener.local_addr().unwrap());

        Self {
            games: Default::default(),
            listener,
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
