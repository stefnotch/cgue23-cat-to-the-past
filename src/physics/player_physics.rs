use bevy_ecs::prelude::*;
use nalgebra::Vector3;
use rapier3d::{
    control::KinematicCharacterController,
    prelude::{ColliderBuilder, QueryFilter, RigidBodyBuilder},
};

use crate::scene::transform::Transform;

use super::physics_context::{BoxCollider, PhysicsContext, RapierRigidBodyHandle, RigidBody};

#[derive(Component, Default)]
pub struct PlayerCharacterController {
    pub collider_height: f32,
    pub desired_movement: Vector3<f32>,
    pub grounded: bool,
}

pub(super) fn apply_player_character_controller_changes(
    mut commands: Commands,
    mut character_controller_query: Query<
        (Entity, &Transform, &PlayerCharacterController),
        (
            Added<PlayerCharacterController>,
            Without<RigidBody>,
            Without<BoxCollider>,
        ),
    >,
    mut physics_context: ResMut<PhysicsContext>,
) {
    let context = physics_context.as_mut();

    for (entity, transform, player_character_controller) in character_controller_query.iter_mut() {
        let physics_rigid_body = RigidBodyBuilder::kinematic_position_based()
            .translation(transform.position.coords.into())
            .build();

        let handle = context.rigid_bodies.insert(physics_rigid_body);
        let collider =
            ColliderBuilder::capsule_y(player_character_controller.collider_height / 2.0, 0.15)
                .translation(
                    // TODO: understand why this is needed
                    Vector3::new(0.0, player_character_controller.collider_height / 2.0, 0.0),
                );

        context
            .colliders
            .insert_with_parent(collider, handle, &mut context.rigid_bodies);

        commands
            .entity(entity)
            .insert(RapierRigidBodyHandle { handle });
    }
}

pub(super) fn step_character_controllers(
    mut physics_context: ResMut<PhysicsContext>,
    mut query: Query<(
        &mut Transform,
        &mut PlayerCharacterController,
        &RapierRigidBodyHandle,
    )>,
) {
    for (mut transform, mut character_controller, rigid_body_handle) in query.iter_mut() {
        let controller = KinematicCharacterController::default();

        let context = physics_context.as_mut();

        let character_rigid_body = context.rigid_bodies.get(rigid_body_handle.handle).unwrap();

        let character_collider = &context
            .colliders
            .get(character_rigid_body.colliders()[0])
            .unwrap();

        let character_mass = character_rigid_body.mass();

        let mut collisions = vec![];
        let effective_movement = controller.move_shape(
            context.integration_parameters.dt,
            &context.rigid_bodies,
            &context.colliders,
            &context.query_pipeline,
            character_collider.shape(),
            character_collider.position(),
            character_controller.desired_movement * context.integration_parameters.dt,
            QueryFilter::new().exclude_rigid_body(rigid_body_handle.handle),
            |c| collisions.push(c),
        );

        character_controller.grounded = effective_movement.grounded;

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

        let character_body = context
            .rigid_bodies
            .get_mut(rigid_body_handle.handle)
            .unwrap();

        let position = character_body.position();
        let new_position = position.translation.vector + effective_movement.translation;
        character_body.set_next_kinematic_translation(new_position);

        transform.position = new_position.into();
    }
}