use bevy_ecs::prelude::Component;

use crate::level::{FlagId, LevelId};

#[derive(Component, Debug)]
pub struct FlagTrigger {
    pub level_id: LevelId,
    pub flag_id: FlagId,
}
