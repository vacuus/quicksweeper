use bevy::{prelude::*, utils::HashMap};

use crate::server::{GameDescriptor, GameMarker};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct GameRegistry(HashMap<GameMarker, GameDescriptor>);
