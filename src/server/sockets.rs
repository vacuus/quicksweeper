use std::net::{TcpListener, TcpStream};

use bevy::prelude::*;
// use tokio::net::TcpStream;
// use tokio_tungstenite::WebSocketStream;
// use futures_lite::future;

use tungstenite::{Message, WebSocket};

use crate::protocol::ApiEvent;

#[derive(Resource, DerefMut, Deref)]
pub struct OpenPort(TcpListener);

impl OpenPort {
    pub fn generate() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener
            .set_nonblocking(true)
            .expect("could not start server in nonblocking mode");
        println!("server bound to: {}", listener.local_addr().unwrap());
        Self(listener)
    }
}

// #[derive(Component, Deref, DerefMut)]
// pub struct PartialConnection(WebSocket<TcpStream>);

#[derive(thiserror::Error, Debug)]
enum ConnectionUpgradeError {
    #[error("Websocket (tungstenite) error: {0}")]
    Tungstenite(#[from] tungstenite::Error),
    #[error("Deserialization error: {0}")]
    DataFormat(rmp_serde::decode::Error),
    #[error("Data received from websocket is not encoded in binary format")]
    Encoding,
}

impl Connection {
    fn read_partial(&mut self) -> Option<Result<ApiEvent, ConnectionUpgradeError>> {
        let msg = match self.read_message() {
            Ok(msg) => Some(msg),
            Err(tungstenite::Error::Io(_)) => None,
            Err(e) => return Some(Err(e.into())),
        }?;

        match msg {
            Message::Ping(_) | Message::Pong(_) => None,
            Message::Binary(v) => {
                Some(rmp_serde::from_slice(&v).map_err(ConnectionUpgradeError::DataFormat))
            }
            _ => Some(Err(ConnectionUpgradeError::Encoding)),
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Connection(WebSocket<TcpStream>);

#[derive(Component)]
pub struct ConnectionInfo {
    name: String,
}

pub fn receive_connections(listener: Res<OpenPort>, mut commands: Commands) {
    if let Ok((client, _)) = listener.accept() {
        client.set_nonblocking(true).expect(
            "Connection failed for reason: Could not connect to client in nonblocking mode",
        );
        match tungstenite::accept(client) {
            Ok(socket) => {
                commands.spawn((Connection(socket),));
                println!("Connection accepted!")
            }
            Err(msg) => eprintln!("Connection failed for reason: {msg:?}"),
        };
    }
}

pub fn upgrade_connections(
    mut partial_connections: Query<(Entity, &mut Connection), Without<ConnectionInfo>>,
    mut commands: Commands,
) {
    for (id, mut client) in partial_connections.iter_mut() {
        if let Some(result) = client.read_partial() {
            match result {
                Ok(ApiEvent::Greet { name }) => {
                    println!("Connection upgraded! It is now able to become a player");
                    println!("received name {name}");
                    commands.entity(id).insert((ConnectionInfo {name}, ));
                }
                // Ok(_) => {
                //     println!("Connection could not be validated, destroying...");
                //     commands.entity(id).despawn();
                // }
                Err(e) => {
                    println!("Connection could not be validated, destroying...");
                    println!("reason: {e}");
                    commands.entity(id).despawn();

                }
            }
        }
    }
}

pub fn listen_clients(mut clients: Query<(Entity, &mut Connection, &ConnectionInfo)>) {
    for (id, mut client, _) in clients.iter_mut() {
        if let Ok(msg) = client.read_message() {
            println!("From {id:?} received message {msg}")
        }
    }
}
