use std::time::Duration;

use bevy_ecs::prelude::Component;
use game_core::time_manager::level_time::LevelTime;
use scene::transform::Transform;

#[derive(Component)]
pub struct Animation {
    pub start_transform: Transform,
    pub end_transform: Transform,
    pub duration: Duration,
}

#[derive(Component)]
pub struct PlayingAnimation {
    pub animation: Animation,
    pub end_time: LevelTime,
}
