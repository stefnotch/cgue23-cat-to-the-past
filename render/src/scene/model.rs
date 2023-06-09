use std::sync::Arc;

use bevy_ecs::prelude::*;
use nalgebra::{Point3, Vector3};
use scene::transform::Transform;

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
impl Primitive {
    pub(crate) fn intersects_frustum(
        &self,
        frustum_bounding_sphere: &(Vector3<f32>, f32),
        transform: &Transform,
    ) -> bool {
        let self_bounding_sphere_center = transform
            .to_matrix()
            .transform_point(&Point3::from(self.mesh.bounding_sphere.0))
            .coords;
        let self_bounding_sphere_radius = self.mesh.bounding_sphere.1 * transform.scale.max();

        let distance = (frustum_bounding_sphere.0 - self_bounding_sphere_center).norm();
        distance < frustum_bounding_sphere.1 + self_bounding_sphere_radius
    }
}

impl GpuModel {}
