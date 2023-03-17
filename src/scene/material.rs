use crate::scene::texture::Texture;
use bevy_ecs::prelude::Component;
use nalgebra::Vector3;
use std::sync::Arc;

use super::loader::Asset;

// #[derive(Component)]
// pub struct MaterialDescriptorSetHandle {
//     pub descriptor_set: Arc<PersistentDescriptorSet>,
// }

/// references a shader and its inputs
#[derive(Component)]
pub struct Material {
    pub color: Vector3<f32>,
    pub base_color_texture: Option<Arc<Texture>>,
    pub ka: f32,
    pub kd: f32,
    pub ks: f32,
    pub alpha: f32,
    // TODO: Add a shader/pipeline here (we only support one shader for now)
}

#[derive(Component)]
pub struct NewMaterial {
    pub base_color: Vector3<f32>,
    pub base_color_texture: Option<Arc<Texture>>,
    pub normal_texture: Option<Arc<Texture>>,
    pub emissivity: Vector3<f32>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
}

impl Asset for Material {}
