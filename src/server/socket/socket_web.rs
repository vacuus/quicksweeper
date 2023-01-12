use bevy::prelude::Resource;
use crossbeam_channel::Receiver;
use js_sys::Uint8Array;
use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{console, BinaryType, MessageEvent, WebSocket};

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
        socket.set_binary_type(BinaryType::Arraybuffer);

        let (tx, rx) = crossbeam_channel::unbounded();

        let onmessage = Closure::<dyn FnMut(_)>::new(move |msg: MessageEvent| {
            log::info!("Receive closure called received data: {:?}", msg.data());
            if let Err(e) = tx.send(msg) {
                log::error!(
                    "Failed to pass websocket message to the wasm backend due to error: \n{}",
                    e.to_string()
                );
            }
        });
        socket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

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
        let _ = self
            .socket
            .send_with_u8_array(&rmp_serde::to_vec(&msg).unwrap());
    }
}
