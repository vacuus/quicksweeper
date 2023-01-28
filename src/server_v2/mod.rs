use futures_util::SinkExt;
use tokio::{net::TcpListener, runtime::Runtime};
use tokio_tungstenite as tungsten;

use crate::server::MessageError;
use crate::{
    registry::REGISTRY,
    server::{ClientMessage, Greeting, ServerMessage},
};

use self::connection::Connection;
use self::double_channel::DoubleChannel;

mod connection;
mod double_channel;

pub struct Player {
    socket: Connection,
    info: Greeting,
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
            Ok(ClientMessage::Create { game, args }) => (),
            Ok(ClientMessage::ForceLeave) => (),
            Ok(ClientMessage::GameTypes) => {
                self.socket
                    .send_ser(ServerMessage::AvailableGames(
                        REGISTRY.keys().copied().collect(),
                    ))
                    .await;
            }
            Ok(ClientMessage::Games) => (),
            Ok(ClientMessage::Ingame { data }) => {}
            Ok(ClientMessage::Join { game }) => (),
            Err(_) => (),
        }
    }
}

#[allow(dead_code)]
pub fn srv_start(address: String) {
    Runtime::new().unwrap().block_on(srv_main(address))
}

async fn srv_main(address: String) {
    let listener = TcpListener::bind(address).await.unwrap();

    while let Ok((sock, _addr)) = listener.accept().await {
        tokio::spawn(async {
            let sock = Connection(tungsten::accept_async(sock).await?);
            match sock.upgrade().await {
                Ok(mut player) => {
                    player.enter().await;
                    Ok(())
                }
                Err(e) => Err(e),
            }
        });
    }
}
