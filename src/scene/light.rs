use bevy_ecs::component::Component;
use nalgebra::Vector3;

// TODO: calculate approximate light range from attenuation and vice-versa
pub struct Attenuation {
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

#[derive(Component)]
pub struct PointLight {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
    pub attenuation: Attenuation,
}
