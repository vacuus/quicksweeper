use bevy::prelude::*;
use bevy_asset_loader::{AssetCollection, AssetLoader};
use derive_more::Deref;

use crate::{minefield::BlankField, SingleplayerState};

#[derive(AssetCollection)]
pub struct Textures {
    #[asset(path = "textures.png")]
    pub mines: Handle<Image>,
    #[asset(path = "cursor.png")]
    pub cursor: Handle<Image>,
}

#[derive(AssetCollection)]
pub struct Field {
    #[asset(path = "test.field")]
    pub field: Handle<BlankField>,
}

#[derive(Deref)]
pub struct MineTextures(Handle<TextureAtlas>);

impl FromWorld for MineTextures {
    fn from_world(world: &mut World) -> Self {
        println!("init minetextures");
        let atlas = TextureAtlas::from_grid(
            world.resource::<Textures>().mines.clone(),
            Vec2::splat(32.0),
            4,
            3,
        );
        let handle = world.resource_mut::<Assets<TextureAtlas>>().add(atlas);
        println!("init done!");
        MineTextures(handle)
    }
}


impl MineTextures {
    pub fn empty(&self) -> SpriteSheetBundle {
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(9),
            texture_atlas: (*self).clone(),
            ..Default::default()
        }
    }
}

pub struct LoadPlugin;

impl Plugin for LoadPlugin {
    fn build(&self, app: &mut App) {
        
        AssetLoader::new(SingleplayerState::Loading)
            .continue_to_state(SingleplayerState::PreGame)
            .with_collection::<Textures>()
            .with_collection::<Field>()
            .init_resource::<MineTextures>()
            .build(app);

    }
}