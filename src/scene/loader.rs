use bevy_ecs::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

// textures
// meshes
// materials
// lights
pub trait Asset {}

#[derive(Resource)]
pub struct Assets<T: Asset> {
    assets: HashMap<Uuid, T>,
}
