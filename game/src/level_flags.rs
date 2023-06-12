use std::collections::HashMap;

use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::{
    prelude::EventReader,
    system::{Res, ResMut, Resource},
};
use levels::{current_level::NextLevel, level_id::LevelId};
use scene::level::FlagId;
use time::time_manager::{
    game_change::{GameChange, GameChangeHistory, GameChangeHistoryPlugin},
    TimeManager,
};

#[derive(Resource)]
pub struct LevelFlags {
    flags: HashMap<LevelId, Vec<bool>>,
}

impl LevelFlags {
    pub fn new() -> Self {
        Self {
            flags: HashMap::new(),
        }
    }

    pub fn set_count(&mut self, level_id: LevelId, count: usize) {
        let old_value = self.flags.insert(level_id, vec![false; count]);
        assert!(old_value.is_none());
    }

    pub fn set_and_record(
        &mut self,
        level_id: LevelId,
        flag_id: FlagId,
        value: bool,
        game_change_history: &mut GameChangeHistory<FlagChange>,
    ) {
        if self.get(level_id, flag_id) == value {
            return;
        }
        self.set(level_id, flag_id, value);
        game_change_history.add_command(FlagChange {
            level_id,
            flag_id,
            value,
        });
    }

    pub fn record_all(
        &self,
        level_id: LevelId,
        game_change_history: &mut GameChangeHistory<FlagChange>,
    ) {
        let count = self.flags.get(&level_id).unwrap().len();
        for flag_id in 0..count {
            game_change_history.add_command(FlagChange {
                level_id,
                flag_id: flag_id as FlagId,
                value: false,
            });
        }
    }

    /// Internal method
    fn set(&mut self, level_id: LevelId, flag_id: FlagId, value: bool) {
        let flags = self
            .flags
            .get_mut(&level_id)
            .unwrap_or_else(|| panic!("Level {:?} does not exist", level_id));

        flags[flag_id] = value;
    }

    pub fn get(&self, level_id: LevelId, flag_id: FlagId) -> bool {
        self.flags
            .get(&level_id)
            .map(|flags| flags[flag_id])
            .unwrap_or_else(|| {
                panic!(
                    "Flag with given {:?} - {:?} does not exist",
                    level_id, flag_id
                )
            })
    }
}

#[derive(Debug, Clone)]
pub struct FlagChange {
    level_id: LevelId,
    flag_id: FlagId,
    value: bool,
}

impl GameChange for FlagChange {}

fn level_flags_rewind(
    time_manager: Res<TimeManager>,
    mut level_flags: ResMut<LevelFlags>,
    mut history: ResMut<GameChangeHistory<FlagChange>>,
) {
    let commands = history.take_commands_to_apply(&time_manager);

    for command_collection in commands {
        for command in command_collection.commands {
            level_flags.set(command.level_id, command.flag_id, command.value);
        }
    }
}

fn level_flags_start_track(
    mut next_level_events: EventReader<NextLevel>,
    level_flags: Res<LevelFlags>,
    mut history: ResMut<GameChangeHistory<FlagChange>>,
) {
    for next_level_event in next_level_events.iter() {
        level_flags.record_all(next_level_event.level_id, &mut history);
    }
}

pub struct LevelFlagsPlugin;

impl Plugin for LevelFlagsPlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app //
            .with_resource(LevelFlags::new())
            .with_plugin(
                GameChangeHistoryPlugin::<FlagChange>::new()
                    .with_tracker(level_flags_start_track)
                    .with_rewinder(level_flags_rewind),
            );
    }
}
