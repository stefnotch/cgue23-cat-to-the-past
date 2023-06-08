use crate::scene::texture::Texture;
use bevy_ecs::prelude::*;
use std::sync::Arc;

#[derive(Component)]
pub struct GpuUIComponent {
    pub texture: Arc<Texture>,
}
