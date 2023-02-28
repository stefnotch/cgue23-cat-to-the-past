use bevy_ecs::prelude::*;

use super::{material::Material, mesh::Mesh};

#[derive(Component)]
pub struct Model {
    pub mesh: Mesh,
    pub material: Material,
}
