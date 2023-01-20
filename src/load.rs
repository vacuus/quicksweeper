use bevy::{gltf::Gltf, prelude::*};
use bevy_asset_loader::prelude::*;
use iyes_loopless::prelude::AppLooplessStateExt;
use rand::{seq::SliceRandom, Rng};

use crate::{main_menu::Menu, minefield::FieldShape};

#[derive(AssetCollection, Resource)]
pub struct Textures {
    #[asset(texture_atlas(tile_size_x = 32.0, tile_size_y = 32.0, columns = 4, rows = 3))]
    #[asset(path = "textures.png")]
    pub mines: Handle<TextureAtlas>,
    #[asset(path = "Roboto.ttf")]
    pub roboto: Handle<Font>,

    // 3d assets
    #[asset(path = "tiles.glb#Scene0")]
    pub cursor: Handle<Scene>,
    #[asset(path = "tiles.glb#Scene1")]
    pub tile_empty: Handle<Scene>,
    #[asset(path = "tiles.glb#Scene2")]
    pub tile_flagged: Handle<Scene>,

    #[asset(path = "tiles.glb")]
    pub mines_3d: Handle<Gltf>,
}

#[derive(AssetCollection, Resource)]
pub struct Field {
    #[cfg(target_arch = "wasm32")]
    #[asset(key = "fields", collection(typed))]
    pub handles: Vec<Handle<FieldShape>>,
    #[cfg(not(target_arch = "wasm32"))]
    #[asset(path = "fields", collection(typed))]
    pub handles: Vec<Handle<FieldShape>>,
}

impl Field {
    pub fn take_one(&self, rng: &mut impl Rng) -> &Handle<FieldShape> {
        self.handles.choose(rng).unwrap()
    }
}

impl Textures {
    pub fn empty_mine(&self) -> SpriteSheetBundle {
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(9),
            texture_atlas: self.mines.clone(),
            ..Default::default()
        }
    }
}

pub struct ClientLoad;

impl Plugin for ClientLoad {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(Menu::Loading)
                .continue_to_state(Menu::MainMenu)
                .with_dynamic_collections::<StandardDynamicAssetCollection>(vec![
                    "dynamic_asset.assets",
                ])
                .with_collection::<Textures>()
                .with_collection::<Field>(),
        );
    }
}

pub struct ServerLoad;

impl Plugin for ServerLoad {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(Menu::Loading).add_loading_state(
            LoadingState::new(Menu::Loading)
                .continue_to_state(Menu::MainMenu)
                .with_collection::<Field>(),
        );
    }
}
