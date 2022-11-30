use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use rand::{Rng, seq::SliceRandom};

use crate::{
    main_menu::MenuState,
    singleplayer::minefield::{specific::CELL_SIZE, BlankField},
};

#[derive(AssetCollection, Resource)]
pub struct Textures {
    #[asset(path = "textures.png")]
    pub mines: Handle<Image>,
    #[asset(path = "cursor.png")]
    pub cursor: Handle<Image>,
    #[asset(path = "FiraSans-Bold.ttf")]
    pub font: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct Field {
    #[asset(path = "fields", collection(typed))]
    pub handles: Vec<Handle<BlankField>>,
}

impl Field {
    pub fn take_one(&self, rng: &mut impl Rng) -> &Handle<BlankField> {
        self.handles.choose(rng).unwrap()
    }
}

#[derive(Deref, Resource)]
pub struct MineTextures(Handle<TextureAtlas>);

impl FromWorld for MineTextures {
    fn from_world(world: &mut World) -> Self {
        let atlas = TextureAtlas::from_grid(
            world.resource::<Textures>().mines.clone(),
            Vec2::splat(CELL_SIZE),
            4,
            3,
            None,
            None,
        );
        let handle = world.resource_mut::<Assets<TextureAtlas>>().add(atlas);
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
        // AssetLoader::new(MenuState::Loading)
        app.add_loading_state(
            LoadingState::new(MenuState::Loading)
                .continue_to_state(MenuState::MainMenu)
                .with_collection::<Textures>()
                .with_collection::<Field>()
                .init_resource::<MineTextures>(),
        );
    }
}
