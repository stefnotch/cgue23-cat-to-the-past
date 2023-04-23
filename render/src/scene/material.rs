use game_core::asset::{Asset, AssetId};
use nalgebra::Vector3;
use std::sync::Arc;

use super::texture::Texture;

/// references a shader and its inputs
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub id: AssetId,
    pub base_color: Vector3<f32>,
    pub base_color_texture: Option<Arc<Texture>>,
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub emissivity: Vector3<f32>, // TODO: Add a shader/pipeline here (we only support one shader for now)
}

impl Asset for Material {
    fn id(&self) -> AssetId {
        self.id
    }
}
