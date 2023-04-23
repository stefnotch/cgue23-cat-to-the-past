use std::sync::Arc;

use game_core::asset::{Asset, AssetId};
use nalgebra::Vector3;

use crate::texture::CpuTexture;

pub struct CpuMaterial {
    pub id: AssetId,
    pub base_color: Vector3<f32>,
    pub base_color_texture: Option<Arc<CpuTexture>>,
    pub roughness_factor: f32,
    pub metallic_factor: f32,
    pub emissivity: Vector3<f32>,
}

impl Default for CpuMaterial {
    fn default() -> Self {
        Self {
            // TODO: Use a constant ID for the default material
            id: AssetId::new_v4(),
            base_color: Vector3::new(1.0, 0.0, 1.0),
            base_color_texture: None,
            roughness_factor: 1.0,
            metallic_factor: 0.0,
            emissivity: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}

impl Asset for CpuMaterial {
    fn id(&self) -> AssetId {
        self.id
    }
}
