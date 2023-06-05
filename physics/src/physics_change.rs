use std::collections::HashMap;

use bevy_ecs::{
    query::{Changed, Without},
    system::{Query, Res, ResMut, Resource},
    world::Mut,
};
use rapier3d::prelude::RigidBodyType;

use scene::pickup::Pickupable;
use time::time_manager::{
    game_change::{GameChange, GameChangeHistory, InterpolationType},
    TimeManager, TimeState, TimeTracked, TimeTrackedId,
};

use super::physics_context::RigidBody;

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
    mut history: ResMut<GameChangeHistory<RigidBodyTypeChange>>,
    query: Query<(&TimeTracked, &RigidBody), (Changed<RigidBody>, Without<Pickupable>)>,
) {
    for (time_tracked, rigidbody) in &query {
        history.add_command(RigidBodyTypeChange::new(time_tracked, rigidbody.0));
    }
}

#[derive(Resource, Default)]
pub(super) struct RigidBodyTypes {
    pub previous_types: HashMap<TimeTrackedId, RigidBodyType>,
}

// Doesn't handle RigidBodies that have been inserted at runtime
pub(super) fn time_manager_rewind_rigid_body_type(
    time_manager: Res<TimeManager>,
    mut history: ResMut<GameChangeHistory<RigidBodyTypeChange>>,
    mut query: Query<(&TimeTracked, &mut RigidBody)>,
    mut previous_types: ResMut<RigidBodyTypes>,
) {
    match time_manager.time_state() {
        TimeState::Normal => return,
        TimeState::StartRewinding => {
            // We note down the type of the rigid body
            // and then make it kinematic
            for (time_tracked, mut rigidbody) in query.iter_mut() {
                previous_types
                    .previous_types
                    .insert(time_tracked.id(), rigidbody.0);
                rigidbody.0 = RigidBodyType::KinematicPositionBased;
            }
        }
        TimeState::Rewinding => return,
        TimeState::StopRewinding => {
            let mut entities: HashMap<_, Mut<RigidBody>> = query
                .iter_mut()
                .map(|(time_tracked, rigidbody)| (time_tracked.id(), rigidbody))
                .collect();

            // We restore the type of the rigid body
            for old_type in previous_types.previous_types.iter() {
                if let Some(v) = entities.get_mut(&old_type.0) {
                    v.0 = old_type.1.clone();
                }
            }
            previous_types.previous_types.clear();

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
