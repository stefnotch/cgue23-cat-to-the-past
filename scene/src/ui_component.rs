use crate::texture::CpuTexture;
use bevy_ecs::prelude::Component;
use std::sync::Arc;

#[derive(Component)]
pub struct UIComponent {
    pub texture: Arc<CpuTexture>,
    pub depth: f32,
    // width, height is determined by the texture dimensions
    // TODO: maybe make the scale depend on the window size
    pub scale: f32,
    pub angle: f32,
    pub visible: bool,
}
