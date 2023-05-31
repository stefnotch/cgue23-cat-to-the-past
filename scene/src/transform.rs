use bevy_ecs::prelude::*;
use nalgebra::{Isometry, Isometry3, Matrix4, Point3, Quaternion, Unit, UnitQuaternion, Vector3};
use std::ops::{Add, Mul};

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Point3::origin(),
            rotation: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn to_matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::new_translation(&self.position.coords);
        let rotation = Matrix4::from(self.rotation);
        let scaling = Matrix4::new_nonuniform_scaling(&self.scale);

        translation * rotation * scaling
    }

    pub fn to_isometry(&self) -> Isometry<f32, Unit<Quaternion<f32>>, 3> {
        Isometry3::from_parts(self.position.coords.into(), self.rotation)
    }

    fn transform_point(&self, transform_point: &Point3<f32>) -> Point3<f32> {
        let scaled_position: Vector3<f32> = transform_point.coords.component_mul(&self.scale);
        let rotated_position: Vector3<f32> = self.rotation * scaled_position;
        let translated_position: Vector3<f32> = rotated_position.add(&self.position.coords);

        translated_position.into()
    }

    pub fn lerp(&self, other: &Transform, factor: f32) -> Transform {
        // eh, just lerp the components individually
        Transform {
            position: self
                .position
                .coords
                .lerp(&other.position.coords, factor)
                .into(),
            rotation: self.rotation.slerp(&other.rotation, factor),
            scale: self.scale.lerp(&other.scale, factor),
        }
    }
}

impl Mul<Transform> for &Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Self::Output {
        Transform {
            position: self.transform_point(&rhs.position),
            rotation: self.rotation * rhs.rotation,
            scale: self.scale.component_mul(&rhs.scale),
        }
    }
}

#[allow(dead_code)]
pub struct TransformBuilder {
    position: Point3<f32>,
    rotation: UnitQuaternion<f32>,
    scale: Vector3<f32>,
}

#[allow(dead_code)]
impl TransformBuilder {
    pub fn new() -> Self {
        Self {
            position: Point3::origin(),
            rotation: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn position(mut self, position: Point3<f32>) -> Self {
        self.position = position;

        self
    }

    pub fn rotation(mut self, rotation: UnitQuaternion<f32>) -> Self {
        self.rotation = rotation;

        self
    }

    pub fn scale(mut self, scale: Vector3<f32>) -> Self {
        self.scale = scale;

        self
    }

    pub fn build(self) -> Transform {
        Transform {
            position: self.position,
            rotation: self.rotation,
            scale: self.scale,
        }
    }
}
