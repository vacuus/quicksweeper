use serde::{Serialize, Deserialize};

use crate::singleplayer::minefield::FieldShape;


#[derive(Serialize, Deserialize)]
enum ServerUpdate {
    FieldShape(FieldShape),
}

#[derive(Serialize, Deserialize)]
struct ClientUpdate {

}