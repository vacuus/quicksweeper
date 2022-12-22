use std::{
    collections::VecDeque,
    net::{IpAddr, TcpListener, TcpStream},
};

use bevy::prelude::*;
use itertools::Itertools;
use serde::{de::DeserializeOwned, Serialize};
use tungstenite::{
    handshake::{server::NoCallback, MidHandshake},
    HandshakeError, Message, ServerHandshake, WebSocket,
};

use super::Greeting;

#[derive(thiserror::Error, Debug)]
pub enum MessageError {
    #[error("From socket library (tungstenite): {0}")]
    Tungstenite(#[from] tungstenite::Error),
    #[error("From serialization (rmp_serde): {0}")]
    Serialization(#[from] rmp_serde::encode::Error),
    #[error("From deserialization: {0}")]
    Deserialization(rmp_serde::decode::Error),
    #[error("Data received from websocket is not encoded in binary format")]
    Encoding,
}

impl MessageError {
    pub fn disconnected(&self) -> bool {
        use tungstenite::Error;
        match self {
            //TODO: Find all outcomes which suggest the socket cannot be used
            Self::Tungstenite(Error::ConnectionClosed | Error::AlreadyClosed) => true,
            _ => false,
        }
    }
}

#[derive(Resource, DerefMut, Deref)]
pub struct OpenPort(TcpListener);

impl OpenPort {
    pub fn generate(addr: IpAddr) -> Self {
        let listener = TcpListener::bind((addr, 0)).unwrap();
        listener
            .set_nonblocking(true)
            .expect("could not start server in nonblocking mode");
        println!("server bound to: {}", listener.local_addr().unwrap());
        Self(listener)
    }
}

#[derive(Resource, Component)]
pub struct Connection {
    socket: WebSocket<TcpStream>,
    repeat_buffer: VecDeque<Vec<u8>>, // TODO use buffer in continuous sending system
    trials: u8,
    disconnected: bool,
}

impl Connection {
    pub fn new(socket: WebSocket<TcpStream>) -> Self {
        Self {
            socket,
            repeat_buffer: default(),
            disconnected: false,
            trials: 0,
        }
    }

    pub fn recv_message<D>(&mut self) -> Option<Result<D, MessageError>>
    where
        D: DeserializeOwned,
    {
        let msg = match self.socket.read_message() {
            Ok(msg) => Some(msg),
            Err(tungstenite::Error::Io(_)) => None,
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

    /// Asks the connection to attempt sending the message as long as possible. Message failure will
    /// be recorded in the given logging implementation. In general this should be used for messages
    /// which are sent less frequently and should not be dropped.
    pub fn repeat_send_unchecked(&mut self, msg: impl Serialize) {
        self.repeat_buffer
            .push_back(rmp_serde::to_vec(&msg).unwrap());
    }

    pub fn repetition(&mut self) {
        if let Some(data) = self.repeat_buffer.front().cloned() {
            if let Err(e) = self.try_send_buf(data) {
                if self.trials > 10 {
                    log::error!("Message failed to send for reason: {e}");
                    self.trials = 0;
                } else {
                    self.trials += 1;
                }
            } else {
                self.repeat_buffer.pop_front();
                self.trials = 0;
            }
        }
    }

    /// Asks the connection to send the message immediately, but instead of returning the error it
    /// is logged using the current logging implementation.
    pub fn send_logged(&mut self, msg: impl Serialize) {
        if let Err(e) = self.try_send(msg) {
            log::error!("Message failed to send for reason: {e}");
        }
    }

    pub fn try_send(&mut self, msg: impl Serialize) -> Result<(), MessageError> {
        self.try_send_buf(rmp_serde::to_vec(&msg)?)
    }

    fn try_send_buf(&mut self, msg: Vec<u8>) -> Result<(), MessageError> {
        if let Err(e) = self
            .socket
            .write_message(Message::Binary(msg))
            .map_err(MessageError::from)
        {
            self.disconnected = e.disconnected();
            return Err(e);
        }
        Ok(())
    }
}

#[derive(Component)]
pub struct ConnectionInfo {
    pub username: String,
}

pub fn clean_dead_connections(mut commands: Commands, connections: Query<(Entity, &Connection)>) {
    for (id, connection) in connections.iter() {
        if connection.disconnected {
            commands.entity(id).despawn_recursive()
        }
    }
}

/// Acts on the messages given to connections by [Connection::repeat_send]. It will try a certain
/// number of times to send the message through the connection, and upon failing afterward will log
/// the latest error.
pub fn try_send_repeated(mut connections: Query<&mut Connection>) {
    for mut connection in connections.iter_mut() {
        connection.repetition();
    }
}

pub fn receive_connections(
    listener: Res<OpenPort>,
    mut commands: Commands,
    mut conn_queue: Local<Vec<MidHandshake<ServerHandshake<TcpStream, NoCallback>>>>,
) {
    if let Ok((client, _)) = listener.accept() {
        client.set_nonblocking(true).expect(
            "Connection failed for reason: Could not connect to client in nonblocking mode",
        );
        match tungstenite::accept(client) {
            Ok(socket) => {
                commands.spawn((Connection::new(socket),));
            }
            Err(HandshakeError::Interrupted(handshake)) => conn_queue.push(handshake),
            Err(msg) => eprintln!("Connection failed for reason: {msg:?}"),
        };
    }

    let new_queue = conn_queue
        .drain(..)
        .filter_map(|handshake| match handshake.handshake() {
            Ok(socket) => {
                commands.spawn((Connection::new(socket),));
                None
            }
            Err(HandshakeError::Interrupted(handshake)) => Some(handshake),
            Err(msg) => {
                eprintln!("Connection failed for reason: {msg:?}");
                None
            }
        })
        .collect_vec();

    drop(std::mem::replace(&mut *conn_queue, new_queue));
}

pub fn upgrade_connections(
    mut partial_connections: Query<(Entity, &mut Connection), Without<ConnectionInfo>>,
    mut commands: Commands,
) {
    for (id, mut client) in partial_connections.iter_mut() {
        if let Some(result) = client.recv_message() {
            match result {
                Ok(Greeting { username }) => {
                    println!("Connection upgraded! It is now able to become a player");
                    commands.entity(id).insert((ConnectionInfo { username },));
                }
                Err(e) => {
                    println!("Connection could not be validated, destroying...");
                    println!("reason: {e}");
                    commands.entity(id).despawn();
                }
            }
        }
    }
}
