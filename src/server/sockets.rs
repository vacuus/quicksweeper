use std::{
    net::{TcpListener, TcpStream},
    ops::DerefMut,
};

use bevy::prelude::*;
use tungstenite::{Message, WebSocket};

use super::protocol::{ClientData, ClientMessage, ServerData, ServerMessage};

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
    pub fn generate() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener
            .set_nonblocking(true)
            .expect("could not start server in nonblocking mode");
        println!("server bound to: {}", listener.local_addr().unwrap());
        Self(listener)
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Connection(WebSocket<TcpStream>);

pub trait MessageSocket: DerefMut<Target = WebSocket<TcpStream>> {
    fn read_data(&mut self) -> Option<Result<ClientData, MessageError>> {
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
    fn write_data(&mut self, msg: ClientData) -> Result<(), MessageError> {
        Ok(self.write_message(Message::Binary(rmp_serde::to_vec(&msg)?))?)
    }
}

impl<T> MessageSocket for T where T: DerefMut<Target = WebSocket<TcpStream>> {}

#[derive(Component)]
pub struct ConnectionInfo {
    pub username: String,
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
        if let Some(result) = client.read_data() {
            match result {
                Ok(ClientData::Greet { username }) => {
                    println!("Connection upgraded! It is now able to become a player");
                    println!("received name {username}");
                    commands.entity(id).insert((ConnectionInfo { username },));
                }
                #[allow(unreachable_patterns)] // this will no longer be true later
                Ok(_) => {
                    println!("Improper connection negotiation, destroying...");
                    commands.entity(id).despawn();
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

pub fn communicate_clients(
    mut clients: Query<(Entity, &mut Connection), (With<ConnectionInfo>, Without<Parent>)>,
    mut incoming: EventWriter<ClientMessage>,
    mut outgoing: ResMut<Events<ServerMessage>>,
    mut outgoing_collection: Local<Vec<ServerMessage>>,
) {
    outgoing_collection.extend(outgoing.drain());

    for (sender, mut client) in clients.iter_mut() {
        // drain_filter my beloved
        let mut ix = 0;
        while ix < outgoing_collection.len() {
            if outgoing_collection[ix].receiver == sender {
                let msg = outgoing_collection.swap_remove(ix).data;
                let _ = client.write_message(Message::Binary(rmp_serde::to_vec(&msg).unwrap()));
            }
            ix += 1;
        }

        if let Ok(msg) = client.read_message() {
            'normal: {
                let Message::Binary(content) = msg else { break 'normal };
                let Ok(data) = rmp_serde::from_slice(&content) else { break 'normal };
                incoming.send(ClientMessage { sender, data });
                return;
            }
            let _ = client.write_message(Message::Binary(
                rmp_serde::to_vec(&ServerData::Malformed).unwrap(),
            ));
        }
    }
}
