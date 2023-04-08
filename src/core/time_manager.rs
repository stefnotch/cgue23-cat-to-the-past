pub mod game_change;
pub mod level_time;

use std::time::Duration;

use bevy_ecs::{
    prelude::{Component, Entity},
    query::Changed,
    system::{Commands, Query, Res, ResMut, Resource},
};
use winit::event::MouseButton;

use crate::{input::input_map::InputMap, scene::transform::Transform};

use self::{
    game_change::{GameChanges, GameState, SingleFrameGameChanges, StateLookup},
    level_time::LevelTime,
};

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
    /// - have a "budget" for every level, so you can only rewind for a certain amount of time
    /// - have a countdown for every level
    /// - only save actual changes, so when the user is AFK, we don't save anything
    /// - have a max size and remove the oldest commands,
    ///   this is especially useful when it's always possible to restart the level simply by walking back to the beginning
    changes: GameChanges,
    current_frame_commands: SingleFrameGameChanges,
    will_rewind_next_frame: bool,
    is_rewinding: bool,
    level_time: LevelTime,
}

pub fn is_rewinding(time_manager: Res<TimeManager>) -> bool {
    time_manager.is_rewinding
}

impl TimeManager {
    pub fn new() -> Self {
        Self {
            changes: GameChanges::new(),
            current_frame_commands: SingleFrameGameChanges::new(),
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
                self.apply_game_changes(StateLookup::Nearest(self.level_time));
            }
        } else {
            self.level_time = (self.level_time - delta).max(LevelTime::zero());
            // Apply interpolated time
            self.apply_game_changes(StateLookup::Interpolated(self.level_time));
        }
        self.is_rewinding = self.will_rewind_next_frame;
        self.current_frame_commands.set_timestamp(self.level_time);
    }

    pub fn end_frame(&mut self) {
        // Swap the current frame commands with an empty one
        let current_commands = std::mem::replace(
            &mut self.current_frame_commands,
            SingleFrameGameChanges::new(),
        );

        self.changes.add_all(current_commands);
    }

    pub fn add_state<T>(&mut self, id: &TimeTracked, state: T)
    where
        T: GameState + 'static,
    {
        assert!(!self.is_rewinding, "Cannot add states while rewinding");
        self.current_frame_commands.add_state(id.id, state);
    }

    /// Usually called when a next level is started
    pub fn reset(&mut self) {
        self.changes.clear()
    }

    pub fn level_time_seconds(&self) -> f32 {
        self.level_time.as_secs_f32()
    }

    //TODO: Assert that we don't have any leftover current_frame_commands
    pub fn next_level(&mut self) {
        self.level_time = LevelTime::zero();
        self.reset();
    }

    pub fn is_rewinding(&self) -> bool {
        self.is_rewinding
    }

    fn apply_game_changes(&self, level_time_lookup: StateLookup) {
        self.changes.apply(level_time_lookup);
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

pub fn time_manager_track_transform(
    mut time_manager: ResMut<TimeManager>,
    query: Query<(&TimeTracked, &Transform), Changed<Transform>>,
) {
    for (time_tracked, transform) in &query {
        time_manager.add_state(time_tracked, TransformState::new(transform.clone()));
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

struct TransformState {
    value: Transform,
}

impl TransformState {
    fn new(transform: Transform) -> Self {
        Self { value: transform }
    }
}

impl GameState for TransformState {
    fn interpolate(&self, other: &Self, t: f32) -> Self
    where
        Self: Sized,
    {
        Self {
            // TODO: Properly interpolate
            value: self.value.clone(),
        }
    }

    fn apply(&self, entity: &mut bevy_ecs::world::EntityMut) {
        let mut entity_transform = entity.get_mut::<Transform>().unwrap();
        entity_transform.position = self.value.position;
        entity_transform.rotation = self.value.rotation;
        entity_transform.scale = self.value.scale;
    }

    fn skip_during_rewind(&self) -> bool {
        false
    }
}
