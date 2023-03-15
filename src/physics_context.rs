use crate::camera::Camera;
use crate::player::Player;
use crate::scene::transform::Transform;
use crate::time::Time;
use bevy_ecs::prelude::{Added, Component, Query, Res, ResMut, Resource};
use bevy_ecs::query::Without;
use nalgebra::UnitQuaternion;
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
}

pub fn step_physics_simulation(mut physics_context: ResMut<PhysicsContext>, time: Res<Time>) {
    let time = time.as_ref();

    physics_context.step_simulation(time);
}

#[derive(Component)]
pub struct RapierRigidBody {
    // We could refactor that to use two separate components, but this will do for now
    pub handle: Option<RigidBodyHandle>,
}

// for now colliders are created once and never changed or deleted
#[derive(Component)]
pub struct BoxCollider {
    pub size: Vector3<f32>,
}

#[derive(Component)]
pub struct CharacterController {
    pub handle: Option<RigidBodyHandle>,
}

pub fn insert_collider_component(
    camera: Res<Camera>,
    mut physics_context: ResMut<PhysicsContext>,
    box_collider_query: Query<
        (&BoxCollider, &Transform),
        (Added<BoxCollider>, Without<RapierRigidBody>),
    >,
    mut rigid_body_query: Query<
        (&mut RapierRigidBody, &BoxCollider, &Transform),
        Added<RapierRigidBody>,
    >,
    mut character_controller_query: Query<&mut CharacterController, Added<CharacterController>>,
) {
    for (collider, transform) in &box_collider_query {
        let half_size: Vector3<f32> = collider.size * 0.5;
        let physics_collider = ColliderBuilder::cuboid(half_size.x, half_size.y, half_size.z)
            // TODO: scaled colliders are not supported yet
            .position(transform.to_isometry())
            .build();

        physics_context.colliders.insert(physics_collider);
    }

    for (mut rigid_body, collider, transform) in rigid_body_query.iter_mut() {
        let physics_rigid_body = RigidBodyBuilder::dynamic()
            .position(transform.to_isometry())
            .build();

        let context = physics_context.as_mut();
        let handle = context.rigid_bodies.insert(physics_rigid_body);

        let half_size: Vector3<f32> = collider.size * 0.5;
        let physics_collider =
            ColliderBuilder::cuboid(half_size.x, half_size.y, half_size.z).build();

        context
            .colliders
            .insert_with_parent(physics_collider, handle, &mut context.rigid_bodies);

        rigid_body.handle = Some(handle);
    }

    for mut character_controller in character_controller_query.iter_mut() {
        let physics_rigid_body = RigidBodyBuilder::kinematic_position_based()
            .translation(camera.position.coords)
            .build();

        let context = physics_context.as_mut();
        let handle = context.rigid_bodies.insert(physics_rigid_body);
        let collider = ColliderBuilder::capsule_y(0.3, 0.15);

        context
            .colliders
            .insert_with_parent(collider, handle, &mut context.rigid_bodies);
        character_controller.handle = Some(handle);
    }
}

pub fn step_character_controller(
    // TODO: Referencing the camera in the physics part is a bit odd
    mut camera: ResMut<Camera>,
    mut player: ResMut<Player>,
    mut physics_context: ResMut<PhysicsContext>,
    // TODO: No transform, hmm
    query: Query<&CharacterController>,
) {
    for character_controller in &query {
        let controller = KinematicCharacterController::default();
        let context = physics_context.as_mut();

        let character_body = context
            .rigid_bodies
            .get(character_controller.handle.unwrap())
            .unwrap();
        let character_collider = &context
            .colliders
            .get(character_body.colliders()[0])
            .unwrap();
        let character_mass = character_body.mass();

        let mut collisions = vec![];
        let effective_movement = controller.move_shape(
            context.integration_parameters.dt,
            &context.rigid_bodies,
            &context.colliders,
            &context.query_pipeline,
            character_collider.shape(),
            character_collider.position(),
            player.desired_movement * context.integration_parameters.dt,
            QueryFilter::new().exclude_rigid_body(character_controller.handle.unwrap()),
            |c| collisions.push(c),
        );

        if effective_movement.grounded {
            player.jump_available = true;
            // player.velocity.y = 0.0;
        }

        for collision in &collisions {
            controller.solve_character_collision_impulses(
                context.integration_parameters.dt,
                &mut context.rigid_bodies,
                &context.colliders,
                &context.query_pipeline,
                character_collider.shape(),
                character_mass,
                collision,
                QueryFilter::new().exclude_rigid_body(character_controller.handle.unwrap()),
            )
        }

        let character_body = context
            .rigid_bodies
            .get_mut(character_controller.handle.unwrap())
            .unwrap();
        let position = character_body.position();
        let new_position = position.translation.vector + effective_movement.translation;
        character_body.set_next_kinematic_translation(new_position);

        camera.position = new_position.into();
    }
}

pub fn update_transform_system(
    physics_context: Res<PhysicsContext>,
    mut query: Query<(&mut Transform, &RapierRigidBody)>,
) {
    for (mut transform, body_handle) in query.iter_mut() {
        let body = match body_handle.handle {
            Some(handle) => physics_context
                .rigid_bodies
                .get(handle)
                .expect("Rigid body not found"),
            None => continue,
        };
        let translation = body.position().translation;
        let rotation = body.rotation().into_inner();

        transform.translation = translation;
        transform.rotation = UnitQuaternion::from_quaternion(rotation);
    }
}
