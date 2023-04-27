pub mod game_change;
pub mod level_time;

use crate::time::{Time, TimePluginSet};
use std::time::Duration;

use crate::events::NextLevel;
use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::{
    prelude::{Component, EventReader, Events},
    schedule::{IntoSystemConfig, SystemSet},
    system::{Res, ResMut, Resource},
};

use self::level_time::LevelTime;

#[derive(Component)]
pub struct TimeTracked {
    id: uuid::Uuid,
}

pub type TimeTrackedId = uuid::Uuid;

impl TimeTracked {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
        }
    }
    pub fn id(&self) -> TimeTrackedId {
        self.id
    }
}

/// The 4 time states to cycle through
#[derive(Debug)]
pub enum TimeState {
    Normal,
    StartRewinding,
    Rewinding,
    StopRewinding,
}

#[derive(Resource)]
pub struct TimeManager {
    current_frame_timestamp: LevelTime,
    pub will_rewind_next_frame: bool,
    time_state: TimeState,
    level_time: LevelTime,
}

pub fn is_rewinding(time_manager: Res<TimeManager>) -> bool {
    time_manager.is_rewinding()
}

impl TimeManager {
    fn new() -> Self {
        Self {
            current_frame_timestamp: LevelTime::zero(),
            will_rewind_next_frame: false,
            time_state: TimeState::Normal,
            level_time: LevelTime::zero(),
        }
    }

    pub fn start_frame(&mut self, delta: Duration) {
        if self.will_rewind_next_frame {
            // Rewinding
            self.level_time = self.level_time.sub_or_zero(delta);
            match self.time_state {
                TimeState::Normal => {
                    self.time_state = TimeState::StartRewinding;
                }
                TimeState::StartRewinding => {
                    self.time_state = TimeState::Rewinding;
                }
                TimeState::Rewinding => {}
                TimeState::StopRewinding => {
                    self.time_state = TimeState::Rewinding;
                }
            }
        } else {
            match self.time_state {
                TimeState::Normal => {
                    self.level_time += delta;
                }
                TimeState::StartRewinding | TimeState::Rewinding => {
                    // Keep level time unchanged and stop interpolating
                    self.time_state = TimeState::StopRewinding;
                }
                TimeState::StopRewinding => {
                    // Keep level time unchanged and stop interpolating
                    self.time_state = TimeState::Normal;
                }
            }
        }

        self.current_frame_timestamp = self.level_time.clone();
    }

    pub fn level_time_seconds(&self) -> f32 {
        self.level_time.as_secs_f32()
    }

    fn next_level(&mut self) {
        self.level_time = LevelTime::zero();
    }

    pub fn is_rewinding(&self) -> bool {
        match self.time_state {
            TimeState::Normal => false,
            TimeState::StartRewinding => true,
            TimeState::Rewinding => true,
            TimeState::StopRewinding => true,
        }
    }

    pub fn time_state(&self) -> &TimeState {
        &self.time_state
    }

    pub fn is_interpolating(&self) -> bool {
        match self.time_state {
            TimeState::Normal => false,
            TimeState::StartRewinding => true,
            TimeState::Rewinding => true,
            TimeState::StopRewinding => false,
        }
    }
}

fn start_frame(time: Res<Time>, mut time_manager: ResMut<TimeManager>) {
    time_manager.start_frame(time.delta());
}

fn next_level(mut time_manager: ResMut<TimeManager>, mut next_level: EventReader<NextLevel>) {
    if next_level.iter().next().is_some() {
        time_manager.next_level();
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TimeManagerPluginSet {
    StartFrame,
}

pub struct TimeManagerPlugin;

impl Plugin for TimeManagerPlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app.with_resource(TimeManager::new())
            .with_system(
                start_frame
                    .in_set(TimeManagerPluginSet::StartFrame)
                    .after(TimePluginSet::UpdateTime),
            )
            .with_resource(Events::<NextLevel>::default())
            .with_system(next_level.in_set(TimeManagerPluginSet::StartFrame));
    }
}
