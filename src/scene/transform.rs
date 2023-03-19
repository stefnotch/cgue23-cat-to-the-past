use bevy_ecs::prelude::*;
use nalgebra::{
    Isometry, Isometry3, Matrix4, Quaternion, Translation3, Unit, UnitQuaternion, Vector3,
};
use std::ops::{Add, Mul};

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Transform {
    pub translation: Translation3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Translation3::identity(),
            rotation: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn to_matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::new_translation(&self.translation.vector);
        let rotation = Matrix4::from(self.rotation);
        let scaling = Matrix4::new_nonuniform_scaling(&self.scale);

        translation * rotation * scaling
    }

    pub fn to_isometry(&self) -> Isometry<f32, Unit<Quaternion<f32>>, 3> {
        Isometry3::from_parts(self.translation, self.rotation)
    }

    fn transform_point(&self, transform_point: &Translation3<f32>) -> Translation3<f32> {
        let scaled_position: Vector3<f32> = transform_point.vector.component_mul(&self.scale);
        let rotated_position: Vector3<f32> = self.rotation * scaled_position;
        let translated_position: Vector3<f32> = rotated_position.add(&self.translation.vector);

        translated_position.into()
    }
}

impl Mul<Transform> for &Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Self::Output {
        Transform {
            translation: self.transform_point(&rhs.translation),
            rotation: self.rotation * rhs.rotation,
            scale: self.scale.component_mul(&rhs.scale),
        }
    }
}

#[allow(dead_code)]
pub struct TransformBuilder {
    translation: Translation3<f32>,
    rotation: UnitQuaternion<f32>,
    scale: Vector3<f32>,
}

#[allow(dead_code)]
impl TransformBuilder {
    pub fn new() -> Self {
        Self {
            translation: Translation3::identity(),
            rotation: UnitQuaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn translation(mut self, translation: Translation3<f32>) -> Self {
        self.translation = translation;

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
            translation: self.translation,
            rotation: self.rotation,
            scale: self.scale,
        }
    }
}
