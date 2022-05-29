use array2d::Array2D;
use bevy::prelude::*;
use derive_more::Deref;
// use itertools::Itertools;
use iyes_loopless::prelude::AppLooplessStateExt;
use rand::prelude::StdRng;
use tap::Tap;

use crate::textures::MineTextures;
use crate::AppState;

#[derive(Clone, Debug, PartialEq)]
pub enum MineCell {
    Empty,
    Mine,
    MarkedEmpty(u8),
    MarkedMine,
}

#[derive(Deref, DerefMut)]
pub struct Minefield(Array2D<MineCell>);

pub fn regenerate_minefield(mut field: ResMut<Minefield>, mut rng: ResMut<StdRng>) {
    // let mut mine_positions = Array2D::filled_with(MineCell::Empty, 10, 10);
    let len = field.num_elements();
    let cols = field.num_columns();

    // generate thirty mines
    rand::seq::index::sample(&mut *rng, len, len * 3 / 10)
        .into_iter()
        .map(|x| (x / cols, x % cols))
        .for_each(|ix| {
            println!("inserted into {ix:?}");
            field[ix] = MineCell::Mine;
        });
}

pub(crate) fn display_minefield(
    mut commands: Commands,
    textures: Res<MineTextures>,
    field: Res<Minefield>,
) {
    let cols = field.num_columns();

    field
        .elements_row_major_iter()
        .enumerate()
        .map(|(ix, x)| ((ix / cols, ix % cols), x))
        .for_each(|((y, x), _)| {
            commands.spawn_bundle(textures.empty().tap_mut(|b| {
                b.transform = Transform {
                    translation: Vec3::new((x * 32) as f32, (y * 32) as f32, 0f32),
                    ..Default::default()
                }
            }));
        });

    // todo!()
}

pub struct MinefieldPlugin;

impl Plugin for MinefieldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Minefield(Array2D::filled_with(MineCell::Empty, 10, 32)))
            .add_startup_system(regenerate_minefield)
            .add_enter_system(AppState::Game, regenerate_minefield)
            .add_enter_system(AppState::Game, display_minefield);
    }
}
