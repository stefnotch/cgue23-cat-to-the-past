use std::sync::Arc;

use bevy_ecs::prelude::*;

use super::{material::Material, mesh::Mesh};

#[derive(Component)]
pub struct Model {
    pub primitives: Vec<Primitive>,
}

/// Why yes, this mirrors whatever gltf does
pub struct Primitive {
    pub mesh: Arc<Mesh>,
    pub material: Arc<Material>,
}
