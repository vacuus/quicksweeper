use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream, IpAddr},
    ops::DerefMut,
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

#[derive(Resource, Deref, DerefMut)]
pub struct ClientSocket(pub WebSocket<TcpStream>);

#[derive(Resource, DerefMut, Deref)]
pub struct OpenPort(TcpListener);

impl OpenPort {
    pub fn generate(addr: &IpAddr) -> Self {
        let ip = local_ip_address::local_ip().unwrap();
        let listener = TcpListener::bind((ip, 0)).unwrap();
        listener
            .set_nonblocking(true)
            .expect("could not start server in nonblocking mode");
        println!("server bound to: {}", listener.local_addr().unwrap());
        Self(listener)
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Connection(WebSocket<TcpStream>);

pub trait MessageSocket<S>: DerefMut<Target = WebSocket<S>>
where
    S: Read + Write,
{
    fn recv_message<D>(&mut self) -> Option<Result<D, MessageError>>
    where
        D: DeserializeOwned,
    {
        let msg = match self.read_message() {
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
    fn send_message(&mut self, msg: impl Serialize) -> Result<(), MessageError> {
        Ok(self.write_message(Message::Binary(rmp_serde::to_vec(&msg)?))?)
    }
}

impl<T, S> MessageSocket<S> for T
where
    T: DerefMut<Target = WebSocket<S>>,
    S: Read + Write,
{
}

#[derive(Component)]
pub struct ConnectionInfo {
    pub username: String,
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
                commands.spawn((Connection(socket),));
                println!("Connection accepted!")
            }
            Err(HandshakeError::Interrupted(handshake)) => conn_queue.push(handshake),
            Err(msg) => eprintln!("Connection failed for reason: {msg:?}"),
        };
    }

    let new_queue = conn_queue
        .drain(..)
        .filter_map(|handshake| match handshake.handshake() {
            Ok(socket) => {
                commands.spawn((Connection(socket),));
                println!("Connection accepted!");
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
                    println!("received name {username}");
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
