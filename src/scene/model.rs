use std::sync::Arc;

use bevy_ecs::prelude::*;

use super::{material::Material, mesh::Mesh};

#[derive(Component)]
pub struct Model {
    pub mesh: Arc<Mesh>,
    pub material: Arc<Material>,
}
