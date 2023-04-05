use nalgebra::Vector3;
use rapier3d::prelude::RigidBodyType;

use crate::core::time_manager::{game_change::GameChange, TimeTracked};

pub(super) struct VelocityChange {
    id: uuid::Uuid,
    linvel: Vector3<f32>,
    angvel: Vector3<f32>,
}

impl VelocityChange {
    pub fn new(time_tracked: &TimeTracked, linvel: Vector3<f32>, angvel: Vector3<f32>) -> Self {
        Self {
            id: time_tracked.id(),
            linvel,
            angvel,
        }
    }
}

impl GameChange for VelocityChange {
    fn is_similar(&self, other: &Self) -> bool
    where
        Self: Sized,
    {
        // TODO: check if the velocity is on the LERP path...
        self.id == other.id && self.linvel == other.linvel && self.angvel == other.angvel
    }
}

pub(super) struct RigidBodyTypeChange {
    id: uuid::Uuid,
    body_type: RigidBodyType,
}

impl RigidBodyTypeChange {
    pub fn new(time_tracked: &TimeTracked, body_type: RigidBodyType) -> Self {
        Self {
            id: time_tracked.id(),
            body_type,
        }
    }
}

impl GameChange for RigidBodyTypeChange {
    fn is_similar(&self, other: &Self) -> bool
    where
        Self: Sized,
    {
        self.id == other.id && self.body_type == other.body_type
    }
}
