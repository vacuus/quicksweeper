
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
enum ApiEvent<'a> {
    Greet {
        name: &'a str,
    }
}