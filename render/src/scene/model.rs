use std::sync::Arc;

use bevy_ecs::prelude::*;
use math::bounding_box::BoundingBox;
use nalgebra::Vector3;

use super::{material::Material, mesh::Mesh};

#[derive(Component, Clone)]
pub struct GpuModel {
    pub primitives: Vec<Primitive>,
}

/// Why yes, this mirrors whatever gltf does
#[derive(Clone)]
pub struct Primitive {
    pub mesh: Arc<Mesh>,
    pub material: Arc<Material>,
}

impl GpuModel {
    pub fn bounding_box(&self) -> BoundingBox<Vector3<f32>> {
        let bounding_box = self
            .primitives
            .iter()
            .map(|primitive| &primitive.mesh.bounding_box)
            .fold(BoundingBox::empty(), |a, b| (a.combine(b)));

        bounding_box
    }
}
