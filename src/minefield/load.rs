use anyhow::anyhow;
use bevy::{
    asset::{AssetLoader, LoadedAsset},
    reflect::TypeUuid,
};
use derive_more::Deref;
use tap::Tap;

use crate::common::Position;

#[derive(TypeUuid, Debug, Deref)]
#[uuid = "2d02b7fb-4718-4073-82c2-80075e688e08"]
pub struct BlankField(Vec<Position>);

impl BlankField {
    pub fn center(&self) -> Option<Position> {
        let max = self.iter().fold(Position::new(0, 0), |acc, item| {
            acc.tap_mut(|acc| {
                acc.x = std::cmp::max(acc.x, item.x);
                acc.y = std::cmp::max(acc.y, item.y);
            })
        });
        let min = self.iter().fold(Position::new(0, 0), |acc, item| {
            acc.tap_mut(|acc| {
                acc.x = std::cmp::min(acc.x, item.x);
                acc.y = std::cmp::min(acc.y, item.y);
            })
        });

        let abs_center = min + (max - min) / 2;

        std::iter::once(abs_center)
            .chain(abs_center.neighbors())
            .filter_map(|pos| self.contains(&pos).then(|| pos))
            .next()
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
            let mut positions = Vec::with_capacity(bytes.len()); // TODO: make loading consistent

            bytes
                .rsplit(|&b| b == b'\n')
                .enumerate()
                .try_for_each(|(y, line)| {
                    line.iter()
                        .enumerate()
                        .try_for_each(|(x, char)| match char {
                            b'x' => {
                                positions.push(Position::new(x as u32, y as u32));
                                Ok(())
                            }
                            b'o' => Ok(()),
                            _ => Err(anyhow!("invalid character")),
                        })
                })?;

            ctx.set_default_asset(LoadedAsset::new(BlankField(positions)));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["field"]
    }
}
