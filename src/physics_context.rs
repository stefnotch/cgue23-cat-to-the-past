use crate::scene::transform::Transform;
use crate::time::Time;
use bevy_ecs::prelude::{Added, Component, Query, Res, ResMut, Resource};
use rapier3d::na::Vector3;
use rapier3d::prelude::{
    BroadPhase, CCDSolver, ColliderBuilder, ColliderSet, ImpulseJointSet, IntegrationParameters,
    IslandManager, MultibodyJointSet, NarrowPhase, PhysicsPipeline, QueryPipeline, Real,
    RigidBodyHandle, RigidBodySet,
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
        self.integration_parameters.dt = (time.delta_seconds as Real) / (self.substeps as Real);

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
}

pub fn step_physics_simulation(mut physics_context: ResMut<PhysicsContext>, time: Res<Time>) {
    let time = time.as_ref();

    physics_context.step_simulation(time);
}

#[derive(Component)]
pub struct RapierRigidBody {
    handle: RigidBodyHandle,
}

// for now colliders are created once and never changed or deleted
#[derive(Component)]
pub struct BoxCollider {
    pub size: Vector3<f32>,
}

pub fn insert_collider_component(
    mut physics_context: ResMut<PhysicsContext>,
    query: Query<&BoxCollider, Added<BoxCollider>>,
) {
    for collider in &query {
        let half_size: Vector3<f32> = collider.size * 0.5;
        let physics_collider =
            ColliderBuilder::cuboid(half_size.x, half_size.y, half_size.z).build();

        physics_context.colliders.insert(physics_collider);
    }
}

pub fn update_transform_system(
    physics_context: Res<PhysicsContext>,
    mut query: Query<(&mut Transform, &RapierRigidBody)>,
) {
    for (mut transform, body_handle) in query.iter_mut() {
        let body = physics_context
            .rigid_bodies
            .get(body_handle.handle)
            .expect("Rigid body not found");

        // TODO: change to nalgebra
        let position = body.position().translation.vector;
        let position: cgmath::Vector3<f32> =
            cgmath::Vector3::new(position.x, position.y, position.z);
        transform.position = position;
        // let rotation = body.rotation()
        // TODO: updaterotation
    }
}
