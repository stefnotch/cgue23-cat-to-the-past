use crate::physics_context::{PhysicsContext, RapierRigidBodyHandle, RigidBody};
use bevy_ecs::prelude::{Added, Component, Query, RemovedComponents, Res, ResMut};
use nalgebra::Point3;
use rapier3d::control::KinematicCharacterController;
use rapier3d::dynamics::RigidBodyType;
use rapier3d::prelude::QueryFilter;
use scene::camera::Camera;
use scene::transform::Transform;

#[derive(Component)]
pub struct PickedUp {
    pub position: Point3<f32>,
}

pub(super) fn start_pickup(mut query: Query<&mut RigidBody, Added<PickedUp>>) {
    for mut rigidbody in query.iter_mut() {
        rigidbody.0 = RigidBodyType::KinematicPositionBased;
    }
}

pub(super) fn stop_pickup(
    mut removals: RemovedComponents<PickedUp>,
    mut query: Query<&mut RigidBody>,
) {
    for entity in &mut removals {
        if let Ok(mut rigidbody) = query.get_mut(entity) {
            rigidbody.0 = RigidBodyType::Dynamic;
        }
    }
}

pub(super) fn update_pickup_target_position(camera: Res<Camera>, mut query: Query<&mut PickedUp>) {
    for mut pickup in query.iter_mut() {
        pickup.position =
            camera.position + camera.orientation * Camera::forward().into_inner() * 3.0
    }
}

pub(super) fn update_pickup_transform(
    mut query: Query<(&mut Transform, &PickedUp, &RapierRigidBodyHandle)>,
    mut physics_context: ResMut<PhysicsContext>,
) {
    let controller = KinematicCharacterController {
        slide: true,
        snap_to_ground: None,
        autostep: None,
        ..Default::default()
    };

    let context = physics_context.as_mut();

    for (mut transform, picked_up, rigid_body_handle) in query.iter_mut() {
        let character_rigid_body = context
            .rigid_bodies
            .get_mut(rigid_body_handle.handle)
            .unwrap();
        character_rigid_body.enable_ccd(true);

        let character_collider = context
            .colliders
            .get(character_rigid_body.colliders()[0])
            .unwrap();

        let current_target_offset = picked_up.position - transform.position;

        let character_mass = character_rigid_body.mass();

        let mut collisions = vec![];
        let effective_movement = controller.move_shape(
            context.integration_parameters.dt,
            &context.rigid_bodies,
            &context.colliders,
            &context.query_pipeline,
            character_collider.shape(),
            character_collider.position(),
            current_target_offset,
            QueryFilter::new()
                .exclude_rigid_body(rigid_body_handle.handle)
                .exclude_sensors(),
            |c| collisions.push(c),
        );

        for collision in &collisions {
            controller.solve_character_collision_impulses(
                context.integration_parameters.dt,
                &mut context.rigid_bodies,
                &context.colliders,
                &context.query_pipeline,
                character_collider.shape(),
                character_mass,
                collision,
                QueryFilter::new().exclude_rigid_body(rigid_body_handle.handle),
            )
        }

        let character_rigid_body = context
            .rigid_bodies
            .get_mut(rigid_body_handle.handle)
            .unwrap();

        character_rigid_body.enable_ccd(true);

        let position = character_rigid_body.position();
        let new_position = position.translation.vector + effective_movement.translation;
        character_rigid_body.set_next_kinematic_translation(new_position);

        transform.position = new_position.into();
    }
}
