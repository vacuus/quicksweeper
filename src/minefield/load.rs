use anyhow::anyhow;
use bevy::{
    asset::{AssetLoader, LoadedAsset},
    reflect::TypeUuid,
};

use crate::common::Position;

#[derive(TypeUuid, Debug)]
#[uuid = "2d02b7fb-4718-4073-82c2-80075e688e08"]
pub struct BlankField(Vec<Position>);

pub struct FieldLoader;

impl AssetLoader for FieldLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        ctx: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async {
            let mut positions = Vec::new(); // TODO: calculate capacity

            bytes
                .rsplit(|&b| b == b'\n')
                .enumerate()
                .try_for_each(|(y, line)| {
                    line.into_iter()
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
