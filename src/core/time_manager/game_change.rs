use std::collections::HashMap;

use bevy_ecs::world::EntityMut;

use super::level_time::LevelTime;

pub trait GameState
where
    Self: Sync + Send,
{
    // Unsure about this one vv
    fn skip_during_rewind(&self) -> bool;

    fn interpolate(&self, other: &Self, t: f32) -> Self
    where
        Self: Sized;

    // Alternate design: this could fire an event which a system picks up on?
    // https://docs.rs/bevy_ecs/latest/bevy_ecs/world/struct.World.html#method.get_entity_mut
    fn apply(&self, entity: &mut EntityMut);
}

pub(super) struct SingleFrameGameChanges {
    timestamp: LevelTime,
    changes: HashMap<uuid::Uuid, GameStateRange>,
}

/// Collections of game changes over time
pub(super) struct GameChanges {
    // Requirements
    // - fast UUID lookups
    // - don't store the timestamp twice for different UUIDs
    // - don't store the UUID twice for different timestamps
    // - fast insertion
    timestamps: Vec<LevelTime>,
    changes: HashMap<uuid::Uuid, GameStateRange>,
}

pub(super) enum StateLookup {
    Nearest(LevelTime),
    Interpolated(LevelTime),
}

impl StateLookup {
    pub fn time(&self) -> LevelTime {
        match self {
            StateLookup::Nearest(time) => *time,
            StateLookup::Interpolated(time) => *time,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct LevelTimeIndex(usize);

/// An object with a unique ID lives for a certain amount of time, and has a state at each point in time
pub(super) struct GameStateRange {
    start_time: LevelTimeIndex,
    // TODO: Add a lifetime here?
    states: Vec<Box<dyn GameState>>,
}

impl GameChanges {
    pub fn new() -> Self {
        Self {
            timestamps: Vec::new(),
            changes: HashMap::new(),
        }
    }

    pub fn add_all(&mut self, changes: SingleFrameGameChanges) {
        let timestamp_index = LevelTimeIndex(self.timestamps.len());
        self.timestamps.push(changes.timestamp);

        for (id, new_state_range) in changes.changes {
            let state_range = self.get_or_create_state_range(id, timestamp_index);

            state_range.states.extend(new_state_range.states);
        }
    }

    fn get_or_create_state_range(
        &mut self,
        id: uuid::Uuid,
        timestamp_index: LevelTimeIndex,
    ) -> &mut GameStateRange {
        self.changes.entry(id).or_insert_with(|| GameStateRange {
            start_time: timestamp_index,
            states: Vec::new(),
        })
    }

    pub fn clear(&mut self) {
        self.timestamps.clear();
        self.changes.clear();
    }

    pub fn apply(&self, level_time_lookup: StateLookup) {
        if self.timestamps.is_empty() {
            return;
        }

        let (time_start_index, time_end) = self.get_time_index_range(level_time_lookup);
    }

    fn get_time_index_range(
        &self,
        level_time_lookup: StateLookup,
    ) -> (usize, Option<(usize, f32)>) {
        if self.timestamps.len() == 1 {
            (0, None)
        } else {
            match self.timestamps.binary_search(&level_time_lookup.time()) {
                Ok(time_index) => (time_index, None),
                Err(time_index) => {
                    if time_index == self.timestamps.len() {
                        (time_index - 1, None)
                    } else if time_index == 0 {
                        (time_index, None)
                    } else {
                        (time_index - 1, Some((time_index, 0.0)))
                    }
                }
            }
        }
    }
}

impl SingleFrameGameChanges {
    pub fn new() -> Self {
        Self {
            timestamp: LevelTime::zero(),
            changes: HashMap::new(),
        }
    }

    pub fn set_timestamp(&mut self, timestamp: LevelTime) {
        self.timestamp = timestamp;
    }

    pub fn add_state<T>(&mut self, id: uuid::Uuid, state: T)
    where
        T: GameState + 'static, // TODO: Seems like the wrong lifetime
    {
        let state_range = self.changes.entry(id).or_insert_with(|| GameStateRange {
            start_time: LevelTimeIndex(0),
            states: Vec::new(),
        });

        state_range.states.push(Box::new(state));
    }
}
