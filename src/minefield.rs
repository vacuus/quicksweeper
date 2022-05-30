use crate::common::Position;
use crate::textures::MineTextures;
use crate::AppState;
use array2d::Array2D;
use bevy::prelude::*;
use derive_more::Deref;
use itertools::Itertools;
use iyes_loopless::prelude::AppLooplessStateExt;
use rand::prelude::StdRng;
use tap::Tap;

#[derive(Clone, Debug)]
pub struct MineCell {
    sprite: Entity,
    state: MineCellState,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MineCellState {
    Empty,
    Mine,
    MarkedEmpty(u8),
    MarkedMine,
}

#[derive(Component)]
struct Mine(Position);

#[derive(Deref, DerefMut, Component)]
pub struct Minefield(Array2D<MineCell>);

fn generate_minefield(
    mut commands: Commands,
    textures: Res<MineTextures>,
    mut rng: ResMut<StdRng>,
) {
    let rows = 10;
    let cols = 20;

    let len = rows * cols;

    // generate mines with density 3/10
    let indices = rand::seq::index::sample(&mut *rng, len, len * 3 / 10);

    let minefield_iter = (0..len).map(|ix| {
        let (y, x) = (ix / cols, ix % cols);

        let state = indices
            .iter()
            .contains(&ix)
            .then(|| MineCellState::Mine)
            .unwrap_or(MineCellState::Empty);

        let sprite = commands
            .spawn_bundle(textures.empty().tap_mut(|b| {
                b.transform = Transform {
                    translation: Vec3::new(x as f32 * 32.0, y as f32 * 32.0, 3.0),
                    ..Default::default()
                };
            }))
            .id();

        MineCell { sprite, state }
    });

    let minefield = Array2D::from_iter_row_major(minefield_iter, rows, cols);

    commands.spawn().insert(Minefield(minefield));
}

pub struct MinefieldPlugin;

impl Plugin for MinefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::Game, generate_minefield);
    }
}
