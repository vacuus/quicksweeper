use bevy::asset::AssetLoader;

use crate::common::Position;

pub struct BlankField(Vec<Position>);

pub struct FieldLoader;

impl AssetLoader for FieldLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        ctx: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async {

            bytes.rsplit(|&b| b == b'\n').for_each(|line| {
                let line = std::str::from_utf8(line).unwrap();
                println!("{line}");
            });

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["field"]
    }
}
