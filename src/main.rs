#![allow(dead_code)]

mod minefield;

use bevy::prelude::*;
use derive_more::Deref;
use iyes_loopless::prelude::AppLooplessStateExt;
use rand::prelude::*;
use tap::Tap;

#[derive(Deref)]
struct MineTextures(Handle<TextureAtlas>);

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
enum AppState {
    Menu,
    Game,
}

fn init(
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
    let atlas = assets.add(TextureAtlas::from_grid(texture, Vec2::splat(32.0), 4, 4));

    commands.insert_resource(MineTextures(atlas));
}

fn main() {
    App::new()
        .add_loopless_state(AppState::Game)
        .insert_resource(StdRng::from_entropy())
        .add_plugins(DefaultPlugins)
        .add_startup_system(init)
        .add_plugin(minefield::MinefieldPlugin)
        .run();
}
