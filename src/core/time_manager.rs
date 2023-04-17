pub mod game_change;
pub mod level_time;
pub mod transform_change;

use std::time::Duration;

use bevy_ecs::{
    prelude::{Component, EventReader, Events},
    schedule::{IntoSystemConfig, Schedule},
    system::{Res, ResMut, Resource},
    world::World,
};
use winit::event::MouseButton;

use crate::{core::time_manager::game_change::GameChangeHistory, input::input_map::InputMap};

use self::{game_change::GameChange, level_time::LevelTime};

use super::{
    application::AppStage,
    events::NextLevel,
    time::{update_time, Time},
};

#[derive(Component)]
pub struct TimeTracked {
    id: uuid::Uuid,
}

impl TimeTracked {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
        }
    }
    pub fn id(&self) -> uuid::Uuid {
        self.id
    }
}

/// The 4 time states to cycle through
pub enum TimeState {
    Normal,
    StartRewinding,
    Rewinding,
    StopRewinding,
}

#[derive(Resource)]
pub struct TimeManager {
    current_frame_timestamp: Option<LevelTime>,
    will_rewind_next_frame: bool,
    time_state: TimeState,
    level_time: LevelTime,
}

pub fn is_rewinding(time_manager: Res<TimeManager>) -> bool {
    time_manager.is_rewinding()
}

impl TimeManager {
    pub fn new() -> Self {
        Self {
            current_frame_timestamp: Some(LevelTime::zero()),
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
                    self.level_time += delta;
                    self.time_state = TimeState::Normal;
                }
            }
        }

        self.current_frame_timestamp = Some(self.level_time.clone());
    }

    pub fn end_frame(&mut self) {
        self.current_frame_timestamp = None;
    }

    pub fn add_command<T>(&mut self, command: T, history: &mut GameChangeHistory<T>)
    where
        T: GameChange,
    {
        assert!(!self.is_rewinding(), "Cannot add commands while rewinding");
        let timestamp = self
            .current_frame_timestamp
            .expect("Cannot add commands outside of a frame");
        history.add_command(timestamp, command);
    }

    pub fn level_time_seconds(&self) -> f32 {
        self.level_time.as_secs_f32()
    }

    pub fn next_level(&mut self) {
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

    // TODO:
    // - Spawn Entity (Commands)
    // - Delete Entity (Commands)
    // - Change entity components (Commands)
    // - Change entity values (mut)

    // TODO:
    // Pop
    // Peek
    // Apply game changes
    // - kinematic character controller
    // - ...

    pub fn setup_systems(self, world: &mut World, schedule: &mut Schedule) {
        world.insert_resource(self);
        schedule.add_system(start_frame.in_set(AppStage::StartFrame).after(update_time));
        schedule.add_system(read_input.in_set(AppStage::Update));
        schedule.add_system(end_frame.in_set(AppStage::EndFrame));

        world.insert_resource(Events::<NextLevel>::default());
        schedule.add_system(next_level.in_set(AppStage::StartFrame));
    }
}

fn start_frame(time: Res<Time>, mut time_manager: ResMut<TimeManager>) {
    time_manager.start_frame(time.delta());
}

fn end_frame(mut time_manager: ResMut<TimeManager>) {
    time_manager.end_frame();
}

fn read_input(mut time_manager: ResMut<TimeManager>, mouse_input: Res<InputMap>) {
    time_manager.will_rewind_next_frame = mouse_input.is_mouse_pressed(MouseButton::Right);
}

fn next_level(mut time_manager: ResMut<TimeManager>, mut next_level: EventReader<NextLevel>) {
    if next_level.iter().next().is_some() {
        time_manager.next_level();
    }
}
