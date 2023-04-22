use crate::scene::texture::Texture;
use bevy_ecs::prelude::Component;
use nalgebra::Vector3;
use std::sync::Arc;

use super::loader::Asset;

/// references a shader and its inputs
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Material {
    pub base_color: Vector3<f32>,
    pub base_color_texture: Option<Arc<Texture>>,
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub emissivity: Vector3<f32>, // TODO: Add a shader/pipeline here (we only support one shader for now)
}

impl Default for Material {
    fn default() -> Self {
        Self {
            base_color: Vector3::new(1.0, 0.0, 1.0),
            base_color_texture: None,
            roughness_factor: 1.0,
            metallic_factor: 0.0,
            emissivity: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}

impl Asset for Material {}
