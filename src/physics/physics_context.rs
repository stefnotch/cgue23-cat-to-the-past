use crate::core::application::AppStage;
use crate::core::time::Time;
use crate::core::time_manager::game_change::GameChangeHistory;
use crate::scene::bounding_box::BoundingBox;
use crate::scene::transform::{Transform, TransformBuilder};
use bevy_ecs::prelude::{
    Added, Commands, Component, Entity, IntoSystemConfig, Query, Res, ResMut, Resource, Schedule,
    With, World,
};
use bevy_ecs::query::Without;
use nalgebra::{Point3, UnitQuaternion};
use rapier3d::na::Vector3;
use rapier3d::prelude::*;

use super::physics_change::{
    time_manager_rewind_rigid_body_type, time_manager_rewind_rigid_body_velocity,
    time_manager_track_rigid_body_type, time_manager_track_rigid_body_velocity,
    RigidBodyTypeChange, VelocityChange,
};
use super::player_physics::{
    apply_player_character_controller_changes, step_character_controllers,
    PlayerCharacterController,
};

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
        // Keep ECS and physics world in sync
        schedule.add_system(apply_collider_changes.in_set(AppStage::Update));
        schedule.add_system(apply_rigid_body_changes.in_set(AppStage::Update));
        schedule.add_system(apply_player_character_controller_changes.in_set(AppStage::Update));

        // Update physics world and write back to ECS world
        schedule.add_system(step_physics_simulation.in_set(AppStage::UpdatePhysics));
        schedule.add_system(step_character_controllers.in_set(AppStage::BeforeRender));
        schedule.add_system(update_transform_system.in_set(AppStage::BeforeRender));
        schedule.add_system(update_move_body_position_system.in_set(AppStage::BeforeRender));

        let velocity_history = GameChangeHistory::<VelocityChange>::new();
        velocity_history.setup_systems(
            world,
            schedule,
            time_manager_track_rigid_body_velocity,
            time_manager_rewind_rigid_body_velocity,
        );

        let body_type_history = GameChangeHistory::<RigidBodyTypeChange>::new();
        body_type_history.setup_systems(
            world,
            schedule,
            time_manager_track_rigid_body_type,
            time_manager_rewind_rigid_body_type,
        );
    }
}

pub fn step_physics_simulation(mut physics_context: ResMut<PhysicsContext>, time: Res<Time>) {
    let time = time.as_ref();

    physics_context.step_simulation(time);
}

#[derive(Component)]
pub struct MoveBodyPosition {
    pub new_position: Point3<f32>,
}

#[derive(Component)]
pub(super) struct RapierRigidBodyHandle {
    pub handle: RigidBodyHandle,
}

#[derive(Component)]
struct RapierColliderHandle {
    handle: ColliderHandle,
}

#[derive(Component)]
pub struct RigidBody(pub RigidBodyType);

// for now colliders are created once and never changed or deleted
#[derive(Component)]
pub struct BoxCollider {
    pub bounds: BoundingBox<Vector3<f32>>,
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
    mut rigid_body_query: Query<(Entity, &BoxCollider, &Transform, &RigidBody), Added<RigidBody>>,
) {
    let context = physics_context.as_mut();

    // Rigid bodies like the cube
    for (entity, collider, transform, RigidBody(body_type)) in rigid_body_query.iter_mut() {
        let physics_rigid_body = RigidBodyBuilder::new(body_type.clone())
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

fn update_transform_system(
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

fn update_move_body_position_system(
    mut physics_context: ResMut<PhysicsContext>,
    query: Query<(&RapierRigidBodyHandle, &MoveBodyPosition), With<RigidBody>>,
) {
    for (rigid_body_handle, MoveBodyPosition { new_position }) in query.iter() {
        let rigid_body = physics_context
            .rigid_bodies
            .get_mut(rigid_body_handle.handle)
            .unwrap();
        rigid_body.set_next_kinematic_translation(new_position.coords);
    }
}
