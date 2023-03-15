use bevy_ecs::component::Component;
use nalgebra::{Point3, Vector3};

// TODO: calculate approximate light range from attenuation and vice-versa
#[derive(Debug)]
pub struct Attenuation {
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

// TODO: remove and replace with light enum
#[derive(Component)]
pub struct PointLight {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
    pub attenuation: Attenuation,
}

#[derive(Component, Debug)]
pub enum Light {
    PointLight {
        position: Point3<f32>,
        color: Vector3<f32>,
        range: f32,
        intensity: f32,
    },
}
