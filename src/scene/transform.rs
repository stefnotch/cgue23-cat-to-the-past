use bevy_ecs::prelude::*;
use nalgebra::{
    Isometry, Isometry3, Matrix4, Point3, Quaternion, Translation3, Unit, UnitQuaternion, Vector3,
};

#[derive(Component)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn to_matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::new_translation(&self.position.coords);
        let rotation = Matrix4::from(self.rotation);
        let scaling = Matrix4::new_nonuniform_scaling(&self.scale);

        translation * rotation * scaling
    }

    pub fn to_isometry(&self) -> Isometry<f32, Unit<Quaternion<f32>>, 3> {
        Isometry3::from_parts(
            Translation3::new(self.position.x, self.position.y, self.position.z),
            self.rotation,
        )
    }
}

pub struct TransformBuilder {
    position: Point3<f32>,
    rotation: UnitQuaternion<f32>,
    scale: Vector3<f32>,
}

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
