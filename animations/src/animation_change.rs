use bevy_ecs::query::Changed;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use bevy_ecs::system::ResMut;
use bevy_ecs::world::Mut;
use levels::current_level::CurrentLevel;
use levels::level_id::LevelId;
use std::collections::HashMap;
use time::time_manager::game_change::GameChange;
use time::time_manager::game_change::GameChangeHistory;
use time::time_manager::level_time::LevelTime;
use time::time_manager::TimeManager;
use time::time_manager::TimeTrackedId;

use crate::animation::PlayingAnimation;

#[derive(Debug, Clone)]
pub struct PlayingAnimationChange {
    pub(crate) id: TimeTrackedId,
    pub(crate) end_time: LevelTime,
    pub(crate) reverse: bool,
}

impl GameChange for PlayingAnimationChange {}

pub(super) fn animations_track(
    mut history: ResMut<GameChangeHistory<PlayingAnimationChange>>,
    current_level: Res<CurrentLevel>,
    query: Query<(&PlayingAnimation, &LevelId), Changed<PlayingAnimation>>,
) {
    for (animation, level_id) in &query {
        if level_id != &current_level.level_id {
            continue;
        }
        history.add_command(PlayingAnimationChange {
            id: animation.id,
            end_time: animation.end_time,
            reverse: animation.reverse,
        });
    }
}

pub(crate) fn animations_rewind(
    time_manager: Res<TimeManager>,
    mut history: ResMut<GameChangeHistory<PlayingAnimationChange>>,
    mut query: Query<&mut PlayingAnimation>,
) {
    let mut entities: HashMap<_, Mut<PlayingAnimation>> = query
        .iter_mut()
        .map(|animation| (animation.id, animation))
        .collect();

    let commands = history.take_commands_to_apply(&time_manager);

    for command_collection in commands {
        for command in command_collection.commands {
            if let Some(v) = entities.get_mut(&command.id) {
                v.end_time = command.end_time;
                v.reverse = command.reverse;
            }
        }
    }
}
