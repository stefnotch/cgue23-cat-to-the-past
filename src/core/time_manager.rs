pub mod game_change;
pub mod level_time;

use std::{collections::VecDeque, time::Duration};

use bevy_ecs::{
    prelude::{Component, Entity},
    query::Changed,
    system::{Commands, Query, Res, ResMut, Resource},
};
use winit::event::MouseButton;

use crate::{input::input_map::InputMap, scene::transform::Transform};

use self::{game_change::GameChange, level_time::LevelTime};

use super::time::Time;

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

#[derive(Resource)]
pub struct TimeManager {
    /// To limit the size of this, we could either
    /// - have a countdown for every level
    /// - only save actual changes, so when the user is AFK, we don't save anything
    /// - have a max size and remove the oldest commands,
    ///   this is especially useful when it's always possible to restart the level simply by walking back to the beginning
    commands: VecDeque<GameChanges>,
    current_frame_commands: GameChanges,
    will_rewind_next_frame: bool,
    is_rewinding: bool,
    level_time: LevelTime,
}

pub fn is_rewinding(time_manager: Res<TimeManager>) -> bool {
    time_manager.is_rewinding
}

/// All game changes in one frame
struct GameChanges {
    timestamp: LevelTime,
    commands: Vec<Box<dyn GameChange>>,
}

impl TimeManager {
    pub fn new() -> Self {
        Self {
            commands: VecDeque::new(),
            current_frame_commands: GameChanges {
                timestamp: LevelTime::zero(),
                commands: Vec::new(),
            },
            will_rewind_next_frame: false,
            is_rewinding: false,
            level_time: LevelTime::zero(),
        }
    }

    pub fn start_frame(&mut self, delta: Duration) {
        if !self.will_rewind_next_frame {
            self.level_time += delta;
            // If we were rewinding in the previous frame
            if self.is_rewinding {
                // Jump to the closest place where you actually have all the required data
                // We gotta be careful with the compression here, wouldn't want funny glitches
            }
        } else {
            self.level_time = (self.level_time - delta).max(LevelTime::zero());
            // Apply undo stack
            self.apply_commands();
            // Apply interpolated time
        }
        self.is_rewinding = self.will_rewind_next_frame;
        self.current_frame_commands.timestamp = self.level_time.clone();
    }

    pub fn end_frame(&mut self) {
        // Swap the current frame commands with an empty one
        let current_commands = std::mem::replace(
            &mut self.current_frame_commands,
            GameChanges {
                timestamp: self.level_time.clone(),
                commands: Vec::new(),
            },
        );

        // Only if any commands were added, add them to the queue
        if current_commands.commands.len() > 0 {
            self.commands.push_back(current_commands);
        }
    }

    pub fn add_command(&mut self, command: Box<dyn GameChange>) {
        assert!(!self.is_rewinding, "Cannot add commands while rewinding");
        self.current_frame_commands.commands.push(command);
    }

    /// Usually called when a next level is started
    pub fn reset(&mut self) {
        self.commands.clear()
    }

    pub fn level_time_seconds(&self) -> f32 {
        self.level_time.as_secs_f32()
    }

    pub fn next_level(&mut self) {
        self.level_time = LevelTime::zero();
        self.reset();
    }

    pub fn is_rewinding(&self) -> bool {
        self.is_rewinding
    }

    fn apply_commands(&mut self) {
        loop {
            if self.commands.len() < 3 {
                // If there's only one element, we can't really rewind time any further
                // If there are only two elements, we don't have to apply any commands, instead we interpolate between them
                return;
            }

            let _top = self.commands.get(self.commands.len() - 1).unwrap();
            let previous = self.commands.get(self.commands.len() - 2).unwrap();

            // If we're further back in the past
            if self.level_time < previous.timestamp {
                // We can pop the top and apply it
                let top = self.commands.pop_back().unwrap();
                //top.apply()
            } else {
                // Nothing to do
                break;
            }
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
}

/// Only tracks translations for now
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

pub fn time_manager_start_frame(
    mut commands: Commands,
    time: Res<Time>,
    mut time_manager: ResMut<TimeManager>,
    query: Query<(Entity, &TimeTracked)>,
) {
    time_manager.start_frame(time.delta());
}

pub fn time_manager_end_frame(mut time_manager: ResMut<TimeManager>) {
    time_manager.end_frame();
}

pub fn time_manager_input(mut time_manager: ResMut<TimeManager>, mouse_input: Res<InputMap>) {
    time_manager.will_rewind_next_frame = mouse_input.is_mouse_pressed(MouseButton::Right);
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

impl GameChange for TransformChange {
    fn is_similar(&self, other: &Self) -> bool
    where
        Self: Sized,
    {
        // TODO: check if the transform is on the LERP path...
        self.id == other.id && self.new_transform == other.new_transform
    }
}
