use std::{collections::HashSet, sync::Mutex};

use bevy_ecs::system::Resource;

use crate::level_id::LevelId;

#[derive(Resource)]
pub struct CurrentLevel {
    pub level_id: LevelId,
    finished_levels: HashSet<LevelId>,
    start_next_level: Mutex<Option<LevelId>>,
}

impl CurrentLevel {
    pub fn new() -> Self {
        Self {
            level_id: LevelId::new(0),
            finished_levels: HashSet::new(),
            start_next_level: Mutex::new(None),
        }
    }

    pub fn start_next_level(&self, level_id: LevelId) {
        println!("start_next_level: {:?}", level_id);
        if self.finished_levels.contains(&level_id) {
            return;
        }
        println!("definitely start_next_level: {:?}", level_id);

        let mut start_next_level = self.start_next_level.lock().unwrap();
        *start_next_level = Some(level_id);
    }

    pub(crate) fn try_start_next_level(&mut self) -> Option<NextLevel> {
        let mut start_next_level = self.start_next_level.lock().unwrap();

        if let Some(level_id) = start_next_level.take() {
            let old_level_id = self.level_id;
            self.finished_levels.insert(old_level_id);

            self.level_id = level_id;
            Some(NextLevel {
                level_id,
                old_level_id,
            })
        } else {
            None
        }
    }
}

pub struct NextLevel {
    pub level_id: LevelId,
    pub old_level_id: LevelId,
}
