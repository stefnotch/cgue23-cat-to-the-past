use bevy_ecs::event::EventWriter;
use game_core::time::Time;

use bevy_ecs::prelude::{
    Added, Commands, Component, DetectChanges, Entity, Query, Res, ResMut, Resource, With,
};
use bevy_ecs::query::{Changed, Without};

use game_core::time_manager::TimeTracked;
use math::bounding_box::BoundingBox;
use nalgebra::{Point3, UnitQuaternion};
use rapier3d::na::Vector3;
pub use rapier3d::prelude::QueryFilter;
pub use rapier3d::prelude::Ray;
use rapier3d::prelude::*;
use scene::transform::{Transform, TransformBuilder};

use super::player_physics::PlayerCharacterController;

use crate::physics_events::{collider2entity, handle_collision_event, CollisionEvent};
use crate::pickup_physics::PickedUp;
pub use rapier3d::prelude::RigidBodyType;

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

            gravity: Vector3::new(0.0, -9.81, 0.0),
            substeps: 1,
        }
    }

    pub fn step_simulation(
        &mut self,
        time: &Time,
        mut collision_events: EventWriter<CollisionEvent>,
    ) {
        self.integration_parameters.dt = (time.delta_seconds() as Real) / (self.substeps as Real);

        let (collision_send, collision_recv) = crossbeam::channel::unbounded();
        let (contact_force_send, contact_force_recv) = crossbeam::channel::unbounded();
        let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

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
            &event_handler,
        );

        while let Ok(collision_event) = collision_recv.try_recv() {
            handle_collision_event(&self.colliders, collision_event, &mut collision_events);
        }

        while let Ok(contact_force_event) = contact_force_recv.try_recv() {
            // Handle the contact force event.
            println!("Received contact force event: {:?}", contact_force_event);
        }
    }

    pub fn cast_ray(
        &self,
        ray: &Ray,
        max_toi: f32,
        solid: bool,
        filter: QueryFilter,
    ) -> Option<(Entity, f32)> {
        let (handle, toi) = self.query_pipeline.cast_ray(
            &self.rigid_bodies,
            &self.colliders,
            ray,
            max_toi,
            solid,
            filter,
        )?;

        Some((collider2entity(&self.colliders, handle), toi))
    }
}

pub(crate) fn step_physics_simulation(
    mut physics_context: ResMut<PhysicsContext>,
    time: Res<Time>,
    collision_events: EventWriter<CollisionEvent>,
) {
    let time = time.as_ref();

    physics_context.step_simulation(time, collision_events);
}

#[derive(Component)]
pub struct Sensor;

#[derive(Component)]
pub struct MovePositionTo {
    pub new_position: Option<Point3<f32>>,
}

#[derive(Component)]
pub(crate) struct RapierRigidBodyHandle {
    pub handle: RigidBodyHandle,
}

#[derive(Component)]
pub(crate) struct RapierColliderHandle {
    handle: ColliderHandle,
}

#[derive(Component)]
pub struct RigidBody(pub RigidBodyType);

// for now colliders are created once and never changed or deleted
#[derive(Component, Clone)]
pub struct BoxCollider {
    pub bounds: BoundingBox<Vector3<f32>>,
}

fn create_box_collider(
    entity: &Entity,
    box_collider: &BoxCollider,
    transform: &Transform,
) -> Collider {
    let scaled_bounds = box_collider.bounds.scale(&transform.scale);
    let half_size: Vector3<f32> = scaled_bounds.size() * 0.5;
    let collider_offset = scaled_bounds.min + half_size;

    ColliderBuilder::cuboid(half_size.x, half_size.y, half_size.z)
        .position(
            transform.to_isometry()
                * Isometry::translation(collider_offset.x, collider_offset.y, collider_offset.z),
        )
        .user_data(entity.to_bits() as u128)
        // .active_collision_types(ActiveCollisionTypes::all())
        .build()
}

pub(crate) fn apply_collider_changes(
    mut commands: Commands,
    mut physics_context: ResMut<PhysicsContext>,
    box_collider_query: Query<
        (Entity, &BoxCollider, &Transform),
        (Added<BoxCollider>, Without<RigidBody>),
    >,
) {
    for (entity, collider, transform) in &box_collider_query {
        let physics_collider = create_box_collider(&entity, collider, transform);
        let handle = physics_context.colliders.insert(physics_collider);
        commands
            .entity(entity)
            .insert(RapierColliderHandle { handle });
    }
}

pub(crate) fn apply_rigid_body_added(
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

        let physics_collider = create_box_collider(&entity, collider, &scale_transform);

        context
            .colliders
            .insert_with_parent(physics_collider, handle, &mut context.rigid_bodies);

        commands
            .entity(entity)
            .insert(RapierRigidBodyHandle { handle });
    }
}

pub(crate) fn apply_rigid_body_type_change(
    mut physics_context: ResMut<PhysicsContext>,
    mut query: Query<(&RigidBody, &RapierRigidBodyHandle), Changed<RigidBody>>,
) {
    for (RigidBody(body_type), RapierRigidBodyHandle { handle }) in query.iter_mut() {
        let rigid_body = physics_context
            .rigid_bodies
            .get_mut(*handle)
            .expect("Rigid body not found");

        // Technically this is uselessly executed when a rigid body is created, but eh
        rigid_body.set_body_type(body_type.clone(), true);
    }
}

pub(crate) fn apply_collider_sensor_change(
    mut physics_context: ResMut<PhysicsContext>,
    mut query: Query<&RapierColliderHandle, With<Sensor>>,
) {
    for RapierColliderHandle { handle } in query.iter_mut() {
        let collider = physics_context
            .colliders
            .get_mut(*handle)
            .expect("Collider not found");

        collider.set_sensor(true);
        collider.set_active_events(ActiveEvents::COLLISION_EVENTS);
    }
}

pub(crate) fn update_transform_system(
    physics_context: Res<PhysicsContext>,
    mut query: Query<
        (&mut Transform, &RapierRigidBodyHandle),
        (Without<PlayerCharacterController>, Without<PickedUp>),
    >,
) {
    for (mut transform, body_handle) in query.iter_mut() {
        if transform.is_changed() {
            // println!("Warning: Transform changed illegally");
        }
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

pub(crate) fn update_move_body_position_system(
    mut physics_context: ResMut<PhysicsContext>,
    mut query: Query<(&RapierRigidBodyHandle, &mut MovePositionTo), With<RigidBody>>,
) {
    for (rigid_body_handle, mut move_body_position) in query.iter_mut() {
        if let Some(position) = move_body_position.new_position {
            let rigid_body = physics_context
                .rigid_bodies
                .get_mut(rigid_body_handle.handle)
                .unwrap();

            rigid_body.set_next_kinematic_translation(position.coords);
            move_body_position.new_position = None;
        }
    }
}
// TODO: time_manager_rewind_transform and update_transform_system already do stuff like this, so we could remove this
// *if* we remove the MoveBodyPosition
pub(crate) fn time_rewinding_move_body_transform(
    mut physics_context: ResMut<PhysicsContext>,
    query: Query<(&RapierRigidBodyHandle, &Transform), (With<RigidBody>, With<TimeTracked>)>,
) {
    for (
        rigid_body_handle,
        Transform {
            position,
            rotation,
            scale: _,
        },
    ) in query.iter()
    {
        // We aren't using the MoveBodyPosition for this, since if we did, we might mess up entities that already have a MoveBodyPosition
        let rigid_body = physics_context
            .rigid_bodies
            .get_mut(rigid_body_handle.handle)
            .unwrap();
        rigid_body.set_next_kinematic_translation(position.coords);
        rigid_body.set_next_kinematic_rotation(rotation.clone());
    }
}
