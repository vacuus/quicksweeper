use bevy::prelude::Resource;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{console, MessageEvent, WebSocket};
use crossbeam_channel::Receiver;

pub struct Connection {
    message_receiver: Receiver<MessageEvent>,
    socket: WebSocket,
}

unsafe impl Send for Connection {}
unsafe impl Sync for Connection {}

impl Connection {
    pub fn new(address: &str) -> Self {
        let socket = WebSocket::new(address).unwrap();

        let (tx, rx) = crossbeam_channel::unbounded();

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
