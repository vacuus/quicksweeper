use bevy::{prelude::*, render::texture::ImageSampler};
use bevy_asset_loader::prelude::*;
use iyes_loopless::prelude::{AppLooplessStateExt, IntoConditionalSystem};
use rand::{seq::SliceRandom, Rng};

use crate::{cursor::ScaleFactor, main_menu::MenuState, singleplayer::minefield::FieldShape};

#[derive(AssetCollection, Resource)]
pub struct Textures {
    #[asset(texture_atlas(tile_size_x = 32.0, tile_size_y = 32.0, columns = 4, rows = 3))]
    #[asset(path = "textures.png")]
    pub mines: Handle<TextureAtlas>,
    #[asset(path = "cursor.png")]
    pub cursor: Handle<Image>,
    #[asset(path = "Roboto.ttf")]
    pub roboto: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct Field {
    #[asset(path = "fields", collection(typed))]
    pub handles: Vec<Handle<FieldShape>>,
}

fn set_texture_mode(
    image_assets: &mut ResMut<Assets<Image>>,
    handle: &Handle<Image>,
    mode: ImageSampler,
) {
    image_assets.get_mut(handle).unwrap().sampler_descriptor = mode;
}

fn set_texture_modes(
    texture_atlas: Res<Assets<TextureAtlas>>,
    mut image: ResMut<Assets<Image>>,
    textures: Res<Textures>,
    scale_factor: Res<ScaleFactor>,
) {
    let mode = if **scale_factor > 1. {
        ImageSampler::linear()
    } else {
        ImageSampler::nearest()
    };

    if scale_factor.is_added() || scale_factor.is_changed() {
        set_texture_mode(
            &mut image,
            &texture_atlas.get(&textures.mines).unwrap().texture,
            mode.clone(),
        );
        set_texture_mode(&mut image, &textures.cursor, mode);
    }
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
        .add_system(set_texture_modes.run_not_in_state(MenuState::Loading));
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
