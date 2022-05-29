use array2d::Array2D;
use bevy::prelude::*;
use derive_more::Deref;
use rand::prelude::StdRng;

pub struct MinefieldPlugin;

impl Plugin for MinefieldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Minefield(Array2D::filled_with(MineCell::Empty, 10, 11)))
            .add_startup_system(regenerate_minefield);
    }
}

#[derive(Clone)]
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
    let rows = field.num_rows();
    let cols = field.num_columns();

    // generate thirty mines
    rand::seq::index::sample(&mut *rng, len, len * 3 / 10)
        .into_iter()
        .map(|x| (x / cols, x % rows))
        .for_each(|ix| {
            field[ix] = MineCell::Mine;
        });
}
