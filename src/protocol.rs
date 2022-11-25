
use serde::{Serialize, Deserialize};

use crate::server::GameDescriptor;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Greet {
        username: String,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Games {
        types: Vec<GameDescriptor>,
        
    }
}