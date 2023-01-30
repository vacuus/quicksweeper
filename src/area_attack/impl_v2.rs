use crate::server_v2::game::{GamemodeInitializer, GameComponents};

pub struct IAreaAttack;

impl GamemodeInitializer for IAreaAttack {
    fn create(&self, _: Vec<u8>) -> GameComponents { // area attack has no params for now
        todo!()
    }
}

impl IAreaAttack {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}
