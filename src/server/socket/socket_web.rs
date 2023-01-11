use std::sync::mpsc::Receiver;

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{console, MessageEvent, WebSocket};

pub struct Connection {
    message_receiver: Receiver<MessageEvent>,
    socket: WebSocket,
}

impl Connection {
    pub fn new(address: &str) -> Self {
        let socket = WebSocket::new(address).unwrap();

        let (tx, rx) = std::sync::mpsc::channel();

        let onmessage = Closure::<dyn FnMut(_)>::new(move |msg: MessageEvent| {
            if let Err(e) = tx.send(msg) {
                console::log_1(
                    &format!(
                        "Failed to pass websocket message to the wasm backend due to error: \n{}",
                        e.to_string()
                    )
                    .into(),
                )
            }
        });
        socket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

        Self {
            message_receiver: rx,
            socket,
        }
    }
}
