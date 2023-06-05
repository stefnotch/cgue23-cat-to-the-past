use bevy_ecs::component::Component;
use nalgebra::Vector3;

#[derive(Debug, Clone, PartialEq)]
pub struct PointLight {
    pub color: Vector3<f32>,
    pub range: f32,
    pub intensity: f32,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub enum Light {
    Point(PointLight),
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: Vector3::new(1.0, 1.0, 1.0),
            range: 20.0,
            intensity: 10.0,
        }
    }
}

#[derive(Component)]
pub struct CastShadow;
