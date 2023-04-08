use nalgebra::Vector3;
use rapier3d::prelude::RigidBodyType;

use crate::core::time_manager::game_change::GameState;

use super::physics_context::{PhysicsContext, RapierRigidBodyHandle};

pub(super) struct VelocityState {
    linvel: Vector3<f32>,
    angvel: Vector3<f32>,
}

impl VelocityState {
    pub fn new(linvel: Vector3<f32>, angvel: Vector3<f32>) -> Self {
        Self { linvel, angvel }
    }
}

impl GameState for VelocityState {
    fn interpolate(&self, other: &Self, t: f32) -> Self
    where
        Self: Sized,
    {
        Self {
            linvel: other.linvel.lerp(&self.linvel, t),
            angvel: other.angvel.lerp(&self.angvel, t),
        }
    }

    fn apply(&self, entity: &mut bevy_ecs::world::EntityMut) {
        todo!()
        /* let rigid_body_handle = entity.get::<RapierRigidBodyHandle>().unwrap();

        let mut physics_context = entity.world().resource_mut::<PhysicsContext>();
        let mut rigidbody = physics_context
            .rigid_bodies
            .get_mut(rigid_body_handle.handle)
            .unwrap();

        rigidbody.set_linvel(self.linvel, false);
        rigidbody.set_angvel(self.angvel, true); */
    }

    fn skip_during_rewind(&self) -> bool {
        // Unsure if this should be true or false
        false
    }
}

#[derive(Clone)]
pub(super) struct RigidBodyTypeState {
    body_type: RigidBodyType,
}

impl RigidBodyTypeState {
    pub fn new(body_type: RigidBodyType) -> Self {
        Self { body_type }
    }
}

impl GameState for RigidBodyTypeState {
    fn interpolate(&self, other: &Self, t: f32) -> Self
    where
        Self: Sized,
    {
        // It's always the current state, until after t > 1
        self.clone()
    }

    fn apply(&self, entity: &mut bevy_ecs::world::EntityMut) {
        todo!()
        /*
        let rigid_body_handle = entity.get::<RapierRigidBodyHandle>().unwrap();

        let mut physics_context = entity.into_world_mut().resource_mut::<PhysicsContext>();
        let mut rigidbody = physics_context
            .rigid_bodies
            .get_mut(rigid_body_handle.handle)
            .unwrap();
        rigidbody.set_body_type(self.body_type, true); */
    }

    fn skip_during_rewind(&self) -> bool {
        true
    }
}
