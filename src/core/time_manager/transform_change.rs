use bevy_ecs::{
    query::Changed,
    system::{Query, ResMut},
};

use crate::scene::transform::Transform;

use super::{game_change::GameChange, TimeManager, TimeTracked};

pub fn time_manager_track_transform(
    mut time_manager: ResMut<TimeManager>,
    query: Query<(&TimeTracked, &Transform), Changed<Transform>>,
) {
    for (time_tracked, transform) in &query {
        time_manager.add_command(Box::new(TransformChange::new(
            time_tracked,
            transform.clone(),
        )));
    }
}

struct TransformChange {
    id: uuid::Uuid,
    new_transform: Transform,
}

impl TransformChange {
    fn new(time_tracked: &TimeTracked, transform: Transform) -> Self {
        Self {
            id: time_tracked.id,
            new_transform: transform,
        }
    }
}

impl GameChange for TransformChange {}
