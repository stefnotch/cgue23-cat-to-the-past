use crate::texture::CpuTexture;
use bevy_ecs::prelude::Component;
use nalgebra::Point3;
use std::sync::Arc;

#[derive(Component)]
pub struct UIComponent {
    pub texture: Arc<CpuTexture>,
    /// The position of the center of the UI component
    /// In 0-1 coordinates, with 0,0 in the top left corner
    /// z is the depth
    pub position: Point3<f32>,
    // width, height is determined by the texture dimensions
    // TODO: maybe make the scale depend on the window size
    pub scale: f32,
    pub angle: f32,
    pub visible: bool,
}
