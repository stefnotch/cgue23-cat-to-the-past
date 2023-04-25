use std::collections::HashMap;

use bevy_ecs::system::Resource;

use super::LevelId;

pub type FlagId = usize;

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

    pub fn set(&mut self, level_id: LevelId, flag_id: FlagId, value: bool) {
        let flags = self
            .flags
            .get_mut(&level_id)
            .expect(&format!("Level {:?} does not exist", level_id));

        flags[flag_id] = value;
    }

    pub fn get(&self, level_id: LevelId, flag_id: FlagId) -> bool {
        self.flags
            .get(&level_id)
            .map(|flags| flags[flag_id])
            .expect(&format!(
                "Flag with given {:?} - {:?} does not exist",
                level_id, flag_id
            ))
    }
}
