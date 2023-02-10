use futures_util::SinkExt;
use simple_logger::SimpleLogger;
use tokio::runtime::Runtime;

use crate::server::MessageError;
use crate::{
    registry::REGISTRY,
    server::{ClientMessage, Greeting, ServerMessage},
};

use self::app::{App, GameStore};
use self::connection::Connection;
use self::double_channel::DoubleChannel;

pub mod app;
mod connection;
pub mod double_channel;
mod fields;
pub mod game;

pub use fields::FIELDS;

pub struct Player {
    socket: Connection,
    info: Greeting,
    game_list: GameStore,
    game_channel: Option<DoubleChannel<Vec<u8>>>,
}

impl Player {
    /// Begin listening to messages
    async fn enter(&mut self) {
        loop {
            if let Some(ref mut recv) = self.game_channel {
                tokio::select! {
                    Some(msg) = recv.recv() => {
                        self.socket.send(msg).await;
                    }
                    Some(maybe_msg) = self.socket.recv_message() => {
                        self.handle_client_message(maybe_msg).await;
                    }
                }
            } else if let Some(msg) = self.socket.recv_message().await {
                self.handle_client_message(msg).await;
            }
        }
    }

    async fn handle_client_message(&mut self, message: Result<ClientMessage, MessageError>) {
        match message {
            Ok(ClientMessage::Create { game, args }) => {
                self.game_channel = self
                    .game_list
                    .create_new(&game, args, self.info.clone())
                    .await;
            }
            Ok(ClientMessage::ForceLeave) => (),
            Ok(ClientMessage::GameTypes) => {
                self.socket
                    .send_ser(ServerMessage::AvailableGames(
                        REGISTRY.keys().copied().collect(),
                    ))
                    .await;
            }
            Ok(ClientMessage::Games) => {
                self.socket
                    .send_ser(ServerMessage::ActiveGames(self.game_list.list().await))
                    .await;
            }
            Ok(ClientMessage::Ingame { data }) => {
                if let Some(chan) = &mut self.game_channel {
                    chan.send(data);
                } else {
                    self.socket.send_ser(ServerMessage::Malformed).await;
                }
            }
            Ok(ClientMessage::Join { game }) => {
                if let Some(game) = self.game_list.join(&game, self.info.clone()).await {
                    self.game_channel = Some(game);
                } else {
                    self.socket.send_ser(ServerMessage::Malformed).await;
                }
            }
            Err(_) => (),
        }
    }
}

#[allow(dead_code)]
pub fn srv_start(address: String) {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .expect("logging framework failed to start");
    Runtime::new().unwrap().block_on(srv_main(address))
}

async fn srv_main(address: String) {
    App::new(address).await.run().await;
}
