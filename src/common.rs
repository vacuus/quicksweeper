use std::mem::MaybeUninit;

#[derive(PartialEq, Clone, Debug)]
pub struct Position(pub u32, pub u32);

pub struct PositionNeighborsIter {
    items: [Position; 8],
    size: u8,
}

impl Iterator for PositionNeighborsIter {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        (self.size != 0).then(|| {
            self.size -= 1;
            self.items[self.size as usize].clone()
        })
    }
}

impl Position {
    pub fn iter_neighbors(&self, max_x: u32, max_y: u32) -> PositionNeighborsIter {
        let (max_x, max_y) = (max_x - 1, max_y - 1);
        let mut size = 0;
        let mut items: [Position; 8] = unsafe { MaybeUninit::uninit().assume_init() };

        // left
        if self.0 != 0 {
            // lower left
            if self.1 != 0 {
                items[size] = Position(self.0 - 1, self.1 - 1);
                size += 1;
            }
            // upper left
            if self.1 != max_y {
                items[size] = Position(self.0 - 1, self.1 + 1);
                size += 1;
            }
            items[size] = Position(self.0 - 1, self.1);
            size += 1;
        }
        // right
        if self.0 != max_x {
            // lower right
            if self.1 != 0 {
                items[size] = Position(self.0 + 1, self.1 - 1);
                size += 1;
            }
            // upper right
            if self.1 != max_y {
                items[size] = Position(self.0 + 1, self.1 + 1);
                size += 1;
            }
            items[size] = Position(self.0 + 1, self.1);
            size += 1;
        }
        // bottom
        if self.1 != 0 {
            items[size] = Position(self.0, self.1 - 1);
            size += 1;
        }
        // top
        if self.1 != max_y {
            items[size] = Position(self.0, self.1 + 1);
            size += 1;
        }

        PositionNeighborsIter {
            items,
            size: size as u8,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CheckCell(pub Position);

#[derive(Clone, Debug)]
pub struct InitCheckCell(pub Position);