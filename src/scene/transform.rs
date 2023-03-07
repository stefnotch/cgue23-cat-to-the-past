use bevy_ecs::prelude::*;
use cgmath::{Matrix4, Quaternion, Vector3, Zero};
use rapier3d::na::{Isometry3, Translation3, UnitQuaternion};

#[derive(Component)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}
impl Transform {
    pub fn new() -> Transform {
        Transform {
            position: Vector3::zero(),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn to_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }

    pub fn to_isometry(
        &self,
    ) -> rapier3d::na::Isometry<f32, rapier3d::na::Unit<rapier3d::na::Quaternion<f32>>, 3> {
        Isometry3::from_parts(
            Translation3::new(self.position.x, self.position.y, self.position.z),
            UnitQuaternion::from_quaternion(rapier3d::na::Quaternion::new(
                self.rotation.s,
                self.rotation.v.x,
                self.rotation.v.y,
                self.rotation.v.z,
            )),
        )
    }
}
