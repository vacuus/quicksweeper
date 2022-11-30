use bevy::prelude::Entity;


pub enum ServerTile {
    /// No one has claimed the tile, and the tile does not contain a mine
    Empty,
    /// A player has claimed the tile
    Owned {
        player: Entity,
    },
    /// There is a mine on this tile, and it has not been revealed
    Mine,
    /// There is a mine on this tile, and it has been revealed
    HardMine,
}

pub enum ClientTile {
    Unknown,
    Owned {
        player: Entity,
        num_neighbors: Option<u8>,
    },
    /// There is a mine on this tile, and it has been revealed
    HardMine,
}