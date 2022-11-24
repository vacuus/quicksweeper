use std::net::{TcpListener, TcpStream};

use bevy::prelude::*;
use tungstenite::WebSocket;

#[derive(Resource, DerefMut, Deref)]
struct OpenPort(TcpListener);

#[derive(Component, Deref, DerefMut)]
struct Connection(WebSocket<TcpStream>);

fn receive_client(listener: Res<OpenPort>, mut commands: Commands) {
    for client in listener.incoming().filter_map(|x| x.ok()) {
        println!("found new connection");
        match tungstenite::accept(client) {
            Ok(socket) => {commands.spawn((Connection(socket),));},
            Err(msg) => eprintln!("Connection failed for reason: {msg:?}")
        };
    }
}
