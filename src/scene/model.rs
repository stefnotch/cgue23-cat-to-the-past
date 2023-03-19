use std::sync::Arc;

use crate::scene::bounding_box::BoundingBox;
use bevy_ecs::prelude::*;
use nalgebra::Vector3;

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

impl Model {
    pub fn bounding_box(&self) -> BoundingBox<Vector3<f32>> {
        let bounding_box = self
            .primitives
            .iter()
            .map(|primitive| &primitive.mesh.bounding_box)
            .fold(BoundingBox::empty(), |a, b| (a.combine(b)));

        bounding_box
    }
}
