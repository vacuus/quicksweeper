use bevy::prelude::*;
use derive_more::Deref;
use tap::Tap;

#[derive(Deref)]
pub struct MineTextures(Handle<TextureAtlas>);

pub fn load_textures(
    mut commands: Commands,
    mut assets: ResMut<Assets<TextureAtlas>>,
    asset_server: ResMut<AssetServer>,
) {
    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d().tap_mut(|bundle| {
            bundle.transform.translation = Vec3::new(0.0, 0.0, 1.0);
        }));

    let texture: Handle<Image> = asset_server.load("textures.png");
    let atlas = assets.add(TextureAtlas::from_grid(texture, Vec2::splat(32.0), 4, 3));

    commands.insert_resource(MineTextures(atlas));
}