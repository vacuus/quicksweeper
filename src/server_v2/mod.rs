use serde::de::DeserializeOwned;
use tokio_tungstenite as tungsten;

use futures_util::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    runtime::Runtime,
};
use tungsten::WebSocketStream;
use tungstenite::{Error, Message};

use crate::server::{ClientMessage, Greeting, MessageError};

use self::double_channel::DoubleChannel;

mod double_channel;

pub fn recv_message<D>(msg: Result<Message, Error>) -> Option<Result<D, MessageError>>
where
    D: DeserializeOwned,
{
    let msg = match msg {
        Ok(msg) => Some(msg),
        Err(e) => return Some(Err(e.into())),
    }?;

    match msg {
        Message::Ping(_) | Message::Pong(_) => None,
        Message::Binary(v) => {
            Some(rmp_serde::from_slice(&v).map_err(MessageError::Deserialization))
        }
        _ => Some(Err(MessageError::Encoding)),
    }
}

struct PartialConnection(WebSocketStream<TcpStream>);
struct Player {
    socket: WebSocketStream<TcpStream>,
    info: Greeting,
    game_channel: Option<DoubleChannel<Vec<u8>>>,
}

impl PartialConnection {
    async fn upgrade(self) -> Result<Player, MessageError> {
        let Self(mut sock) = self;
        loop {
            if let Some(msg) = recv_message(sock.next().await.unwrap()) {
                break match msg {
                    Ok(greeting @ Greeting { .. }) => Ok(Player {
                        socket: sock,
                        info: greeting,
                        game_channel: None,
                    }),
                    Err(e) => Err(e),
                };
            }
        }
    }
}

impl Player {
    /// Begin listening to messages
    async fn enter(&mut self) {
        loop {
            if let Some(ref mut recv) = self.game_channel {
                tokio::select! {
                    Some(msg) = recv.recv() => {
                        self.socket.send(Message::Binary(msg)).await;
                    }
                    Some(maybe_msg) = self.socket.next() => {
                        self.handle_client_message(maybe_msg)
                    }
                }
            } else if let Some(msg) = self.socket.next().await {
                self.handle_client_message(msg)
            }
        }
    }

    fn handle_client_message(&mut self, message: Result<Message, Error>) {
        match recv_message(message) {
            Some(Ok(ClientMessage::Create { game, args })) => (),
            Some(Ok(ClientMessage::ForceLeave)) => (),
            Some(Ok(ClientMessage::GameTypes)) => (),
            Some(Ok(ClientMessage::Games)) => (),
            Some(Ok(ClientMessage::Ingame { data })) => (),
            Some(Ok(ClientMessage::Join { game })) => (),
            Some(Err(_)) => (),
            None => (),
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
            let sock = PartialConnection(tungsten::accept_async(sock).await?);
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
