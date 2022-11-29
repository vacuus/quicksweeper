use std::collections::HashMap;

use bevy::prelude::*;

use crate::server::{GameDescriptor, GameMarker};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct GameRegistry(HashMap<GameMarker, GameDescriptor>);

impl GameRegistry {}
