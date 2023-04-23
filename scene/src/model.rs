use std::sync::Arc;

use bevy_ecs::prelude::*;
use math::bounding_box::BoundingBox;
use nalgebra::Vector3;

use crate::{material::CpuMaterial, mesh::CpuMesh};

/// Due to how upload_models works, as soon as the model has been uploaded to the Gpu,
/// nothing can be changed anymore. No material changes either.
#[derive(Component, Clone)]
pub struct Model {
    pub primitives: Vec<CpuPrimitive>,
}

/// Why yes, this mirrors whatever gltf does
#[derive(Clone)]
pub struct CpuPrimitive {
    pub mesh: Arc<CpuMesh>,
    pub material: Arc<CpuMaterial>,
}

impl Model {
    /// Gets the combined bounding box of all the primitives in the model
    pub fn bounding_box(&self) -> BoundingBox<Vector3<f32>> {
        let bounding_box = self
            .primitives
            .iter()
            .map(|primitive| &primitive.mesh.bounding_box)
            .fold(BoundingBox::empty(), |a, b| (a.combine(b)));

        bounding_box
    }
}
