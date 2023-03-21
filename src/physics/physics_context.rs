use crate::core::application::AppStage;
use crate::core::time::Time;
use crate::scene::bounding_box::BoundingBox;
use crate::scene::transform::{Transform, TransformBuilder};
use bevy_ecs::prelude::{
    Added, Commands, Component, Entity, IntoSystemConfig, Query, Res, ResMut, Resource, Schedule,
    World,
};
use bevy_ecs::query::Without;
use nalgebra::{Translation3, UnitQuaternion};
use rapier3d::control::KinematicCharacterController;
use rapier3d::na::Vector3;
use rapier3d::prelude::*;

#[derive(Resource)]
pub struct PhysicsContext {
    /// controls various aspects of the physics simulation
    pub integration_parameters: IntegrationParameters,

    /// responsible for tracking the set of dynamic rigid-bodies that are still moving
    pub island_manager: IslandManager,

    /// responsible for tying everything together in order to run the physics simulation
    pub physics_pipeline: PhysicsPipeline,

    pub rigid_bodies: RigidBodySet,
    pub colliders: ColliderSet,

    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,

    pub impulse_joints: ImpulseJointSet,
    pub multi_body_joints: MultibodyJointSet,

    // responsible for the resolution of Continuous-Collision-Detection
    pub ccd_solver: CCDSolver,

    /// responsible for efficiently running scene queries e.g., ray-casting, shape-casting
    /// (sweep tests), intersection tests, on all the colliders of the scene.
    pub query_pipeline: QueryPipeline,

    /// can be used to apply arbitrary rules to ignore collision detection between some pairs of
    /// colliders
    pub physics_hooks: (),

    /// can be used to get notified when two non-sensor colliders start/stop having contacts, and
    /// when one sensor collider and one other collider start/stop intersecting
    pub event_handler: (),

    pub gravity: Vector3<Real>,

    pub substeps: u32,
}

impl PhysicsContext {
    pub fn new() -> PhysicsContext {
        PhysicsContext {
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            physics_pipeline: PhysicsPipeline::new(),
            rigid_bodies: RigidBodySet::default(),
            colliders: ColliderSet::default(),

            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),

            impulse_joints: ImpulseJointSet::new(),
            multi_body_joints: MultibodyJointSet::new(),

            ccd_solver: CCDSolver::new(),

            query_pipeline: QueryPipeline::new(),

            physics_hooks: (),
            event_handler: (),

            gravity: Vector3::new(0.0, -9.81, 0.0),
            substeps: 1,
        }
    }

    pub fn step_simulation(&mut self, time: &Time) {
        self.integration_parameters.dt = (time.delta_seconds() as Real) / (self.substeps as Real);

        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multi_body_joints,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &self.physics_hooks,
            &self.event_handler,
        );
    }

    pub fn setup_systems(self, world: &mut World, schedule: &mut Schedule) {
        world.insert_resource(self);
        schedule.add_system(apply_collider_changes.in_set(AppStage::Update));
        schedule.add_system(apply_rigid_body_changes.in_set(AppStage::Update));
        schedule.add_system(apply_player_character_controller_changes.in_set(AppStage::Update));

        schedule.add_system(step_physics_simulation.in_set(AppStage::UpdatePhysics));
        schedule.add_system(step_character_controllers.in_set(AppStage::PostUpdate));
        schedule.add_system(update_transform_system.in_set(AppStage::PostUpdate));
    }
}

pub fn step_physics_simulation(mut physics_context: ResMut<PhysicsContext>, time: Res<Time>) {
    let time = time.as_ref();

    physics_context.step_simulation(time);
}

#[derive(Component)]
pub struct RapierRigidBodyHandle {
    pub handle: RigidBodyHandle,
}

#[derive(Component)]
pub struct RapierColliderHandle {
    pub handle: ColliderHandle,
}

#[derive(Component)]
pub struct RigidBody;

// for now colliders are created once and never changed or deleted
#[derive(Component)]
pub struct BoxCollider {
    pub bounds: BoundingBox<Vector3<f32>>,
}

#[derive(Component, Default)]
pub struct PlayerCharacterController {
    pub height: f32,
    pub desired_movement: Vector3<f32>,
    pub grounded: bool,
}

pub fn create_box_collider(box_collider: &BoxCollider, transform: &Transform) -> Collider {
    let scaled_bounds = box_collider.bounds.scale(&transform.scale);
    let half_size: Vector3<f32> = scaled_bounds.size() * 0.5;
    let collider_offset = scaled_bounds.min + half_size;

    ColliderBuilder::cuboid(half_size.x, half_size.y, half_size.z)
        .position(
            transform.to_isometry()
                * Isometry::translation(collider_offset.x, collider_offset.y, collider_offset.z),
        )
        .build()
}

pub fn apply_collider_changes(
    mut commands: Commands,
    mut physics_context: ResMut<PhysicsContext>,
    box_collider_query: Query<
        (Entity, &BoxCollider, &Transform),
        (Added<BoxCollider>, Without<RigidBody>),
    >,
) {
    for (entity, collider, transform) in &box_collider_query {
        let physics_collider = create_box_collider(&collider, &transform);
        let handle = physics_context.colliders.insert(physics_collider);
        commands
            .entity(entity)
            .insert(RapierColliderHandle { handle });
    }
}

pub fn apply_rigid_body_changes(
    mut commands: Commands,
    mut physics_context: ResMut<PhysicsContext>,
    mut rigid_body_query: Query<(Entity, &BoxCollider, &Transform), Added<RigidBody>>,
) {
    let context = physics_context.as_mut();

    // Rigid bodies like the cube
    for (entity, collider, transform) in rigid_body_query.iter_mut() {
        let physics_rigid_body = RigidBodyBuilder::dynamic()
            .position(transform.to_isometry())
            .build();

        let handle = context.rigid_bodies.insert(physics_rigid_body);

        let scale_transform = TransformBuilder::new().scale(transform.scale).build();

        let physics_collider = create_box_collider(&collider, &scale_transform);

        context
            .colliders
            .insert_with_parent(physics_collider, handle, &mut context.rigid_bodies);

        commands
            .entity(entity)
            .insert(RapierRigidBodyHandle { handle });
    }
}

pub fn apply_player_character_controller_changes(
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
        let collider = ColliderBuilder::capsule_y(player_character_controller.height / 2.0, 0.15)
            .translation(
                // TODO: understand why this is needed
                Vector3::new(0.0, player_character_controller.height / 2.0, 0.0),
            );

        context
            .colliders
            .insert_with_parent(collider, handle, &mut context.rigid_bodies);

        commands
            .entity(entity)
            .insert(RapierRigidBodyHandle { handle });
    }
}

pub fn step_character_controllers(
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

pub fn update_transform_system(
    physics_context: Res<PhysicsContext>,
    mut query: Query<(&mut Transform, &RapierRigidBodyHandle), Without<PlayerCharacterController>>,
) {
    for (mut transform, body_handle) in query.iter_mut() {
        let body = physics_context
            .rigid_bodies
            .get(body_handle.handle)
            .expect("Rigid body not found");

        let translation = body.position().translation.vector.into();
        let rotation = body.rotation().into_inner();

        transform.position = translation;
        transform.rotation = UnitQuaternion::from_quaternion(rotation);
    }
}
