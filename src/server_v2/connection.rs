use futures_util::{Sink, SinkExt, Stream, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use crate::server::MessageError;

use super::{Player, app::GameList};

pub struct Connection(pub WebSocketStream<TcpStream>);

impl Connection {
    pub async fn recv_message<D>(&mut self) -> Option<Result<D, MessageError>>
    where
        D: DeserializeOwned,
    {
        let msg = match self.0.next().await.unwrap() {
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

    pub async fn send_ser<S>(&mut self, message: S) -> Result<(), MessageError>
    where
        S: Serialize,
    {
        self.send(rmp_serde::to_vec(&message)?).await?;
        Ok(())
    }

    pub async fn upgrade(mut self, list: GameList) -> Result<Player, MessageError> {
        loop {
            if let Some(msg) = self.recv_message().await {
                break match msg {
                    Ok(greeting) => Ok(Player {
                        socket: self,
                        info: greeting,
                        game_list: list,
                        game_channel: None,
                    }),
                    Err(e) => Err(e),
                };
            }
        }
    }
}

impl Stream for Connection {
    type Item = Result<Message, tungstenite::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx)
    }
}

impl Sink<Vec<u8>> for Connection {
    type Error = tungstenite::Error;

    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready_unpin(cx)
    }

    fn start_send(mut self: std::pin::Pin<&mut Self>, item: Vec<u8>) -> Result<(), Self::Error> {
        self.0.start_send_unpin(Message::Binary(item))
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_flush_unpin(cx)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_close_unpin(cx)
    }
}
