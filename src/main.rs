use bevy::{prelude::*, ecs::system::Command};
use derive_more::Deref;
use tap::Tap;

#[derive(Deref)]
struct MineTextures(Handle<TextureAtlas>);

impl Command for MineTextures {
    fn write(self, world: &mut World) {
        world.insert_resource(self)
    }
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
    let atlas = assets.add(TextureAtlas::from_grid(texture, Vec2::splat(32.0), 4, 3));

    commands.spawn_bundle(SpriteSheetBundle {
        sprite: TextureAtlasSprite::new(0),
        texture_atlas: atlas,
        ..Default::default()
    });
    println!("inserted mine textures")
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(init)
        .run();
}
