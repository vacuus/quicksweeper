use serde::de::DeserializeOwned;
use tokio_tungstenite as tungsten;

use futures_lite::StreamExt;
use tokio::{
    net::{TcpListener, TcpStream},
    runtime::Runtime, sync::mpsc,
};
use tungsten::WebSocketStream;
use tungstenite::{Error, Message};

use crate::server::{ClientMessage, ConnectionInfo, Greeting, MessageError, ServerMessage};

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
                    }),
                    Err(e) => Err(e),
                };
            }
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
                Ok(player) => {

                    Ok(())},
                Err(e) => Err(e),
            }
        });
    }
}
