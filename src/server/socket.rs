use std::net::TcpStream;

use bevy::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use tungstenite::WebSocket;

use super::MessageError;

pub mod socket_pc;
pub mod socket_web;

#[derive(Resource)]
pub enum CommonConnection {
    Pc(socket_pc::Connection),
    Web(socket_web::Connection),
}

impl CommonConnection {
    pub fn new_web(address: &str) -> Self {
        Self::Web(socket_web::Connection::new(address))
    }

    pub fn new_pc(inner_socket: WebSocket<TcpStream>) -> Self {
        Self::Pc(socket_pc::Connection::new(inner_socket))
    }

    pub fn recv_message<D>(&mut self) -> Option<Result<D, MessageError>>
    where
        D: DeserializeOwned,
    {
        match self {
            CommonConnection::Pc(sock) => sock.recv_message(),
            CommonConnection::Web(sock) => sock.recv_message(),
        }
    }

    pub fn send_logged(&mut self, msg: impl Serialize) {
        match self {
            CommonConnection::Pc(sock) => sock.send_logged(msg),
            CommonConnection::Web(sock) => sock.send(msg),
        }
    }

    pub fn repetition(&mut self) {
        if let Self::Pc(sock) = self {
            sock.repetition()
        }
    }

    pub fn try_send(&mut self, msg: impl Serialize) -> Result<(), MessageError> {
        match self {
            CommonConnection::Pc(sock) => sock.try_send(msg),
            CommonConnection::Web(sock) => Ok(sock.send(msg)),
        }
    }
    pub fn repeat_send_unchecked(&mut self, msg: impl Serialize) {
        match self {
            CommonConnection::Pc(sock) => sock.repeat_send_unchecked(msg),
            CommonConnection::Web(sock) => sock.send(msg),
        }
    }

    pub fn is_ready(&self) -> bool {
        match self {
            CommonConnection::Pc(_) => true,
            CommonConnection::Web(sock) => sock.is_ready(),
        }
    }
}
