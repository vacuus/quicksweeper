use std::collections::HashMap;

use bevy::prelude::*;
use once_cell::sync::Lazy;

use crate::{server::{GameDescriptor, GameMarker}, area_attack::{AREA_ATTACK_MARKER, IAreaAttack}};

pub static REGISTRY: Lazy<GameRegistry> = Lazy::new(|| {
    GameRegistry(
        [(
            AREA_ATTACK_MARKER,
            GameDescriptor {
                name: "Area Attack".to_string(),
                description: "Race to claim the board for yourself".to_string(),
                initializer: IAreaAttack::new()
            },
        )]
        .into_iter()
        .collect(),
    )
});

#[derive(Resource, Default, Deref, DerefMut)]
pub struct GameRegistry(HashMap<GameMarker, GameDescriptor>);

impl GameRegistry {}
