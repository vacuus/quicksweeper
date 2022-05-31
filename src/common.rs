
#[derive(PartialEq, Clone, Debug)]
pub struct Position(pub u32, pub u32);

#[derive(Clone, Debug)]
pub struct CheckCell(pub Position);