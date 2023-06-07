use crate::scene::texture::Texture;
use bevy_ecs::prelude::*;
use std::sync::Arc;

#[derive(Component)]
pub struct UIComponent {
    pub texture: Arc<Texture>,
    pub depth: f32,
    // width, height is determined by the texture dimensions
    // TODO: maybe make the scale depend on the window size
    pub scale: f32,
    pub angle: f32,
    pub visible: bool,
}
