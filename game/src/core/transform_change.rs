use std::collections::HashMap;

use bevy_ecs::{
    prelude::EventReader,
    query::Changed,
    system::{Query, Res, ResMut},
};

use levels::{
    current_level::{CurrentLevel, NextLevel},
    level_id::LevelId,
};
use scene::transform::Transform;

use time::time_manager::{
    game_change::{GameChange, GameChangeHistory},
    TimeManager, TimeTracked,
};

// TODO: Am not sure if this is the best place for this code.

pub fn time_manager_track_transform(
    mut history: ResMut<GameChangeHistory<TransformChange>>,
    current_level: Res<CurrentLevel>,
    query: Query<(&TimeTracked, &Transform, &LevelId), Changed<Transform>>,
) {
    for (time_tracked, transform, level_id) in &query {
        if level_id != &current_level.level_id {
            continue;
        }
        history.add_command(TransformChange::new(time_tracked, transform.clone()));
    }
}
pub fn time_manager_start_track_transform(
    mut next_level_events: EventReader<NextLevel>,
    mut history: ResMut<GameChangeHistory<TransformChange>>,
    query: Query<(&TimeTracked, &Transform, &LevelId)>,
) {
    for next_level_event in next_level_events.iter() {
        for (time_tracked, transform, level_id) in &query {
            if level_id != &next_level_event.level_id {
                continue;
            }
            history.add_command(TransformChange::new(time_tracked, transform.clone()));
        }
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

#[derive(Debug, Clone)]
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
