use anyhow::anyhow;
use bevy::prelude::*;
use bevy::{
    asset::{AssetLoader, LoadedAsset},
    reflect::TypeUuid,
};
use itertools::Itertools;
use tap::Tap;

use crate::common::Position;

#[derive(Debug, PartialEq, Clone, Copy)]
enum TileKind {
    Void,
    Exists,
}

impl TileKind {
    fn flip(&self) -> Self {
        match self {
            Self::Void => Self::Exists,
            Self::Exists => Self::Void,
        }
    }
}

impl TryFrom<u8> for TileKind {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'o' | b' ' => Ok(Self::Void),
            b'x' => Ok(Self::Exists),
            _ => Err(anyhow!("Imposter cell detected :bangbang:")),
        }
    }
}

#[derive(Debug)]
enum FieldCode {
    Row { starts: TileKind },
    Run(u32),
}

#[derive(TypeUuid, Debug)]
#[uuid = "2d02b7fb-4718-4073-82c2-80075e688e08"]
pub struct FieldShape(Vec<FieldCode>);

impl FieldShape {
    pub fn decode(&self) -> impl Iterator<Item = Position> + '_ {
        self.0
            .iter()
            .scan(
                (-1_isize, 0_isize, TileKind::Void),
                |(row, col, kind), code| {
                    Some(match code {
                        FieldCode::Row { starts } => {
                            *kind = starts.flip();
                            *row += 1;
                            *col = 0;
                            None
                        }
                        FieldCode::Run(size) => {
                            let this_col = *col;
                            *kind = kind.flip();
                            *col += *size as isize;
                            (*kind == TileKind::Exists).then_some((*row, this_col, *size as isize))
                        }
                    })
                },
            )
            .flatten()
            .flat_map(|(row, col, size)| {
                (col..(col + size)).map(move |col| Position { x: col, y: row })
            })
    }

    pub fn center(&self) -> Option<Position> {
        // let max = self.iter().fold(Position::new(0, 0), |acc, item| {
        //     acc.tap_mut(|acc| {
        //         acc.x = std::cmp::max(acc.x, item.x);
        //         acc.y = std::cmp::max(acc.y, item.y);
        //     })
        // });
        // let min = self.iter().fold(Position::new(0, 0), |acc, item| {
        //     acc.tap_mut(|acc| {
        //         acc.x = std::cmp::min(acc.x, item.x);
        //         acc.y = std::cmp::min(acc.y, item.y);
        //     })
        // });

        // let abs_center = min + (max - min) / 2;

        // std::iter::once(abs_center)
        //     .chain(abs_center.neighbors())
        //     .find_map(|pos| self.contains(&pos).then_some(pos))
        // todo!()
        None
    }
}

pub struct FieldLoader;

impl AssetLoader for FieldLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        ctx: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async {
            let mut data = Vec::new();

            bytes.rsplit(|&b| b == b'\n').try_for_each(|line| {
                let mut prev_kind = line
                    .first()
                    .map(|&c| TileKind::try_from(c))
                    .unwrap_or(Ok(TileKind::Void))?;
                let mut finished = false;

                data.push(FieldCode::Row { starts: prev_kind });

                line.iter()
                    .batching(|it| {

                        if finished {
                            return None
                        }
                        let mut count = 1;

                        loop {
                            match it.next().map(|&e| TileKind::try_from(e)) {
                                Some(Ok(t)) if t == prev_kind => count += 1,
                                Some(Ok(u)) => {
                                    prev_kind = u;
                                    break;
                                }
                                Some(Err(e)) => return Some(Err(e)),
                                None => {
                                    finished = true;
                                    break;
                                }
                            }
                        }

                        Some(Ok(FieldCode::Run(count)))
                    })
                    .try_for_each(|e| e.map(|run| data.push(run)))
            })?;

            println!("generated board {data:?}");

            ctx.set_default_asset(LoadedAsset::new(FieldShape(data)));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["field"]
    }
}
