use bevy_ecs::component::Component;
use nalgebra::Vector3;

#[derive(Debug)]
pub struct PointLight {
    pub color: Vector3<f32>,
    pub range: f32,
    pub intensity: f32,
}

#[derive(Component, Debug)]
pub enum Light {
    Point(PointLight),
}
