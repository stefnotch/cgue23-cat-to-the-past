use bevy_ecs::prelude::Component;
use nalgebra::Vector3;

use super::loader::Asset;

// #[derive(Component)]
// pub struct MaterialDescriptorSetHandle {
//     pub descriptor_set: Arc<PersistentDescriptorSet>,
// }

/// references a shader and its inputs
#[derive(Component)]
pub struct Material {
    pub color: Vector3<f32>,
    pub ka: f32,
    pub kd: f32,
    pub ks: f32,
    pub alpha: f32,
    // TODO: Add a shader/pipeline here (we only support one shader for now)
}

impl Asset for Material {}
