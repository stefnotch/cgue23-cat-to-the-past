use std::sync::Mutex;

use bevy_ecs::system::Resource;

use crate::level_id::LevelId;

#[derive(Resource)]
pub struct CurrentLevel {
    pub level_id: LevelId,

    start_next_level: Mutex<Option<LevelId>>,
}

impl CurrentLevel {
    pub fn new() -> Self {
        Self {
            level_id: LevelId::new(0),
            start_next_level: Mutex::new(None),
        }
    }

    pub fn start_next_level(&self, level_id: LevelId) {
        let mut start_next_level = self.start_next_level.lock().unwrap();
        *start_next_level = Some(level_id);
    }

    pub(crate) fn take_start_next_level(&self) -> Option<LevelId> {
        let mut start_next_level = self.start_next_level.lock().unwrap();
        start_next_level.take()
    }
}

pub struct NextLevel(pub LevelId);
