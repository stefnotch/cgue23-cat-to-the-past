use std::collections::HashMap;

use bevy_ecs::{
    query::Changed,
    system::{Query, Res, ResMut},
    world::Mut,
};
use nalgebra::Vector3;
use rapier3d::prelude::RigidBodyType;

use game_core::time_manager::{
    game_change::{GameChange, GameChangeHistory, InterpolationType},
    TimeManager, TimeState, TimeTracked, TimeTrackedId,
};

use super::physics_context::{PhysicsContext, RapierRigidBodyHandle, RigidBody};

pub(super) struct VelocityChange {
    id: TimeTrackedId,
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

impl GameChange for VelocityChange {}

pub(super) fn time_manager_track_rigid_body_velocity(
    physics_context: Res<PhysicsContext>,
    mut time_manager: ResMut<TimeManager>,
    mut history: ResMut<GameChangeHistory<VelocityChange>>,
    query: Query<(&TimeTracked, &RapierRigidBodyHandle)>,
) {
    for (time_tracked, rigid_body_handle) in &query {
        let rigidbody = physics_context
            .rigid_bodies
            .get(rigid_body_handle.handle)
            .unwrap();

        // Probably a valid optimization to skip sleeping bodies.
        if rigidbody.is_sleeping() {
            continue;
        }

        time_manager.add_command(
            VelocityChange::new(
                time_tracked,
                rigidbody.linvel().clone(),
                rigidbody.angvel().clone(),
            ),
            &mut history,
        );
    }
}

pub(super) fn time_manager_rewind_rigid_body_velocity(
    mut physics_context: ResMut<PhysicsContext>,
    time_manager: Res<TimeManager>,
    mut history: ResMut<GameChangeHistory<VelocityChange>>,
    query: Query<(&TimeTracked, &RapierRigidBodyHandle)>,
) {
    // The code below makes it kinematic
    let entities: HashMap<_, _> = query
        .into_iter()
        .map(|(time_tracked, handle)| (time_tracked.id(), handle))
        .collect();

    let (commands, _interpolation) =
        history.take_commands_to_apply(&time_manager, InterpolationType::None);
    for command_collection in commands {
        for command in command_collection.commands {
            if let Some(v) = entities.get(&command.id) {
                let rigidbody = physics_context
                    .rigid_bodies
                    .get_mut(v.handle)
                    .expect("RigidBody not found in physics context");
                rigidbody.set_linvel(command.linvel, false);
                rigidbody.set_angvel(command.angvel, true);
            }
        }
    }

    // TODO: Interpolation logic
}

pub(super) struct RigidBodyTypeChange {
    id: TimeTrackedId,
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

impl GameChange for RigidBodyTypeChange {}

pub(super) fn time_manager_track_rigid_body_type(
    mut time_manager: ResMut<TimeManager>,
    mut history: ResMut<GameChangeHistory<RigidBodyTypeChange>>,
    query: Query<(&TimeTracked, &RigidBody), Changed<RigidBody>>,
) {
    for (time_tracked, rigidbody) in &query {
        time_manager.add_command(
            RigidBodyTypeChange::new(time_tracked, rigidbody.0),
            &mut history,
        );
    }
}

// TODO: I'm not sure if this is the most elegant way to do this.
pub(super) fn time_manager_rewind_rigid_body_type(
    mut time_manager: ResMut<TimeManager>,
    mut history: ResMut<GameChangeHistory<RigidBodyTypeChange>>,
    mut query: Query<(&TimeTracked, &mut RigidBody)>,
) {
    match time_manager.time_state() {
        TimeState::Normal => return,
        TimeState::StartRewinding => {
            // We note down the type of the rigid body
            // and then make it kinematic
            for (time_tracked, mut rigidbody) in query.iter_mut() {
                time_manager.add_rewinder_command(
                    RigidBodyTypeChange::new(time_tracked, rigidbody.0),
                    &mut history,
                );

                rigidbody.0 = RigidBodyType::KinematicPositionBased;
            }
        }
        TimeState::Rewinding => return,
        TimeState::StopRewinding => {
            let mut entities: HashMap<_, Mut<RigidBody>> = query
                .iter_mut()
                .map(|(time_tracked, rigidbody)| (time_tracked.id(), rigidbody))
                .collect();

            let (commands, _interpolation) =
                history.take_commands_to_apply(&time_manager, InterpolationType::None);
            for command_collection in commands {
                for command in command_collection.commands {
                    if let Some(v) = entities.get_mut(&command.id) {
                        v.0 = command.body_type;
                    }
                }
            }
        }
    }
}