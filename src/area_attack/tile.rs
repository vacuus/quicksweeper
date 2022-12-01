use bevy::prelude::*;

#[derive(Component)]
pub enum ServerTile {
    /// No one has claimed the tile, and the tile does not contain a mine
    Empty,
    /// A player has claimed the tile
    Owned { player: Entity },
    /// There is a mine on this tile, and it has not been revealed
    Mine,
    /// There is a mine on this tile, and it has been revealed
    HardMine,
}

#[derive(Component)]
pub enum ClientTile {
    /// No one has claimed this tile, and it isn't known whether it is blank or contains a mine
    Unknown,
    /// This tile has been claimed by the player specified by the given ID. In addition, if the
    /// client using this type is the one that owns this tile, it will know the number of cells
    /// neighboring this tile which have mines (`num_neighbors`), which can also be zero, in which
    /// case a filled tile without numbers will be shown. If this tile is not owned by the client,
    /// then this field will always be zero.
    Owned { player: Entity, num_neighbors: u8 },
    /// There is a mine on this tile, and it has been revealed
    HardMine,
}
