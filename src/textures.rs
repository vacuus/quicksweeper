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
            bundle.transform.translation = Vec3::new(0.0, 0.0, 100.0);
        }));

    let texture: Handle<Image> = asset_server.load("textures.png");
    let atlas = assets.add(TextureAtlas::from_grid(texture, Vec2::splat(32.0), 4, 3));

    commands.insert_resource(MineTextures(atlas));
}

impl MineTextures {
    pub fn filled_with(&self, amount: u8) -> SpriteSheetBundle {
        assert!(amount < 9);
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(amount as usize),
            texture_atlas: (*self).clone(),
            ..Default::default()
        }
    }

    pub fn empty(&self) -> SpriteSheetBundle {
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(9),
            texture_atlas: (*self).clone(),
            ..Default::default()
        }
    }

    pub fn flagged(&self) -> SpriteSheetBundle {
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(10),
            texture_atlas: (*self).clone(),
            ..Default::default()
        }
    }

    pub fn mine(&self) -> SpriteSheetBundle {
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(11),
            texture_atlas: (*self).clone(),
            ..Default::default()
        }
    }
}
