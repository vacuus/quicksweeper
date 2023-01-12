use bevy::prelude::Resource;
use js_sys::Uint8Array;
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{console, MessageEvent, WebSocket};
use crossbeam_channel::Receiver;

use crate::server::MessageError;

pub struct Connection {
    message_receiver: Receiver<MessageEvent>,
    socket: WebSocket,
}

// TODO I hate my life
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

    pub fn is_ready(&self) -> bool {
        self.socket.ready_state() == 1
    }

    pub fn recv_message<D>(&mut self) -> Option<Result<D, MessageError>>
    where
        D: DeserializeOwned,
    {
        self.message_receiver.try_recv().ok().map(|m_event| {
            let data = Uint8Array::new(&m_event.data()).to_vec();
            rmp_serde::from_slice(&data).map_err(|e| e.into())
        })
    }

    pub fn send(&mut self, msg: impl Serialize) {
        // TODO get diagnostics for this
        let _ = self.socket.send_with_u8_array(&rmp_serde::to_vec(&msg).unwrap());
    }
}
