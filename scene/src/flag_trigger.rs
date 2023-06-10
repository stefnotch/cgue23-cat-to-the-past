use bevy_ecs::prelude::Component;
use levels::level_id::LevelId;

use crate::level::FlagId;

#[derive(Component, Debug)]
pub struct FlagTrigger {
    pub level_id: LevelId,
    pub flag_id: FlagId,
}
