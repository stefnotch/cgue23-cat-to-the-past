use std::collections::HashMap;

use bevy_ecs::{
    query::Changed,
    system::{Query, Res, ResMut},
};

use scene::transform::Transform;

use time::time_manager::{
    game_change::{GameChange, GameChangeHistory},
    TimeManager, TimeTracked,
};

// TODO: Am not sure if this is the best place for this code.

pub fn time_manager_track_transform(
    mut history: ResMut<GameChangeHistory<TransformChange>>,
    query: Query<(&TimeTracked, &Transform), Changed<Transform>>,
) {
    for (time_tracked, transform) in &query {
        history.add_command(TransformChange::new(time_tracked, transform.clone()));
    }
}

pub fn time_manager_rewind_transform(
    time_manager: Res<TimeManager>,
    mut history: ResMut<GameChangeHistory<TransformChange>>,
    mut query: Query<(&TimeTracked, &mut Transform)>,
) {
    let mut entities: HashMap<_, _> = query
        .iter_mut()
        .map(|(time_tracked, transform)| (time_tracked.id(), transform))
        .collect();

    let commands = history.take_commands_to_apply(&time_manager);

    for command_collection in commands {
        for command in command_collection.commands {
            if let Some(v) = entities.get_mut(&command.id) {
                (v.as_mut()).clone_from(&command.new_transform);
            }
        }
    }

    // TODO: Interpolation logic
}

pub struct TransformChange {
    id: uuid::Uuid,
    new_transform: Transform,
}

impl TransformChange {
    fn new(time_tracked: &TimeTracked, transform: Transform) -> Self {
        Self {
            id: time_tracked.id(),
            new_transform: transform,
        }
    }
}

impl GameChange for TransformChange {}
