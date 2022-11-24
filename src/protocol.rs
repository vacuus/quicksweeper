
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ApiEvent {
    Greet {
        name: String,
    }
}