use std::{collections::HashMap, sync::Arc};

use itertools::Itertools;
use tokio::{
    net::TcpListener,
    sync::{
        mpsc::{channel, Receiver, Sender},
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

use super::{connection::Connection, double_channel::DoubleChannel, game::SessionObjects};

pub fn player_connector_pair() -> (GameConnector, PlayerReceiver) {
    let (greeting_tx, greeting_rx) = channel(1);
    let (connector_tx, connector_rx) = channel(1);

    (
        GameConnector {
            request: greeting_tx,
            recv: connector_rx,
        },
        PlayerReceiver {
            recv: greeting_rx,
            reply: connector_tx,
            needs_reply: false,
        },
    )
}

pub struct GameConnector {
    request: Sender<Greeting>,
    recv: Receiver<DoubleChannel<Vec<u8>>>,
}

/// The other end of [GameConnector], held by the game
pub struct PlayerReceiver {
    recv: Receiver<Greeting>,
    reply: Sender<DoubleChannel<Vec<u8>>>,
    needs_reply: bool,
}

impl PlayerReceiver {
    pub async fn recv(&mut self) -> Option<Greeting> {
        self.recv.recv().await.map(|u| {
            self.needs_reply = true;
            u
        }) // TODO feature result_option_inspect
    }

    pub async fn respond(&mut self) -> Option<DoubleChannel<Vec<u8>>> {
        if self.needs_reply {
            self.needs_reply = false;
            let (ch_out, ch_here) = DoubleChannel::double();
            self.reply.send(ch_out).await;
            Some(ch_here)
        } else {
            None
        }
    }
}

impl GameConnector {
    pub async fn connect(&mut self, player: Greeting) -> Option<DoubleChannel<Vec<u8>>> {
        if self.request.send(player).await.is_ok() {
            self.recv.recv().await
        } else {
            None
        }
    }
}

pub struct GameHandle {
    kind: GameMarker,
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

    pub async fn create_new(
        &self,
        game: &GameMarker,
        args: Vec<u8>,
        info: Greeting,
    ) -> Option<DoubleChannel<Vec<u8>>> {
        if let Some(GameDescriptor { initializer, .. }) = REGISTRY.get(game) {
            let SessionObjects {
                host_channel,
                connector,
                main_task,
            } = initializer.create(args, info);
            let key = self.generator.next_id() as u64; //TODO generator reset? Or some way to prevent crashes

            self.store.write().await.insert(
                key,
                GameHandle {
                    kind: *game,
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
