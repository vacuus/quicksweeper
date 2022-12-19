use bevy::{prelude::*, render::texture::ImageSampler};
use bevy_asset_loader::prelude::*;
use iyes_loopless::prelude::AppLooplessStateExt;
use rand::{seq::SliceRandom, Rng};

use crate::{main_menu::MenuState, singleplayer::minefield::FieldShape};

#[derive(AssetCollection, Resource)]
pub struct Textures {
    #[asset(texture_atlas(tile_size_x = 32.0, tile_size_y = 32.0, columns = 4, rows = 3))]
    #[asset(path = "textures.png")]
    pub mines: Handle<TextureAtlas>,
    #[asset(path = "cursor.png")]
    pub cursor: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct Field {
    #[asset(path = "fields", collection(typed))]
    pub handles: Vec<Handle<FieldShape>>,
}

fn set_texture_mode(image_assets: &mut ResMut<Assets<Image>>, handle: &Handle<Image>) {
    image_assets.get_mut(handle).unwrap().sampler_descriptor = ImageSampler::nearest();
}

fn set_texture_modes(
    texture_atlas: Res<Assets<TextureAtlas>>,
    mut image: ResMut<Assets<Image>>,
    textures: Res<Textures>,
) {
    set_texture_mode(
        &mut image,
        &texture_atlas.get(&textures.mines).unwrap().texture,
    );
    set_texture_mode(&mut image, &textures.cursor);
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
            LoadingState::new(MenuState::Loading)
                .continue_to_state(MenuState::MainMenu)
                .with_collection::<Textures>()
                .with_collection::<Field>(),
        )
        .add_exit_system(MenuState::Loading, set_texture_modes);
    }
}

pub struct ServerLoad;

impl Plugin for ServerLoad {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(MenuState::Loading)
            .add_loading_state(
                LoadingState::new(MenuState::Loading)
                    .continue_to_state(MenuState::MainMenu)
                    .with_collection::<Field>(),
            );
    }
}
