use crate::texture::CpuTexture;
use angle::Rad;
use bevy_ecs::prelude::Component;
use nalgebra::{Point2, Point3, Vector2};
use std::sync::Arc;

#[derive(Component)]
pub struct UIComponent {
    /// width, height is determined by the texture dimensions
    pub texture: Arc<CpuTexture>,
    pub texture_position: UITexturePosition,
    /// The position of the UI component
    /// In 0-1 coordinates, with 0,0 in the top left corner of the screen
    /// z is the depth
    pub position: Point3<f32>,
    pub visible: bool,
}

impl UIComponent {
    /// Size in screen pixels
    pub fn get_size(&self) -> Vector2<f32> {
        let texture_size = self.texture.data.dimensions();
        let texture_size = Vector2::new(texture_size[0] as f32, texture_size[1] as f32);

        texture_size.component_mul(&self.texture_position.scale)
    }

    /// Position of the top left corner in screen pixels
    /// z is a depth value, in the range 0-1
    pub fn get_position(&self, screen_size: Vector2<f32>) -> Point3<f32> {
        let position_on_screen = self.position.xy().coords.component_mul(&screen_size);

        let size = self.get_size();
        // e.g. if the texture origin is centered (0.5, 0.5), then this is like "position - half of size"
        let top_left_position =
            position_on_screen - size.component_mul(&self.texture_position.texture_origin.coords);

        Point3::new(top_left_position.x, top_left_position.y, self.position.z)
    }
}

pub struct UITexturePosition {
    /// The size and rotation are relative to the origin
    pub texture_origin: Point2<f32>,
    // TODO: maybe make the scale depend on the window size
    pub scale: Vector2<f32>,
    pub angle: Rad<f32>,
}

impl Default for UITexturePosition {
    fn default() -> Self {
        Self {
            texture_origin: Point2::new(0.0, 0.0),
            scale: Vector2::new(1.0, 1.0),
            angle: Default::default(),
        }
    }
}

impl UITexturePosition {
    pub fn centered() -> Self {
        Self {
            texture_origin: Point2::new(0.5, 0.5),
            ..Default::default()
        }
    }
}
