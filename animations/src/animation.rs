use std::time::Duration;

use bevy_ecs::prelude::Component;
use game_core::time_manager::level_time::LevelTime;
use scene::transform::Transform;

pub struct Animation {
    pub start_transform: Transform,
    pub end_transform: Transform,
    pub duration: Duration,
}

/// An entity with a PlayingAnimation may not be time tracked.
#[derive(Component)]
pub struct PlayingAnimation {
    pub animation: Animation,
    pub end_time: LevelTime,
    /// Also can be used to keep the animation frozen at the start.
    pub reverse: bool,
}

impl PlayingAnimation {
    pub fn get_transform(&self, time: LevelTime) -> Transform {
        let progress = self.get_progress(time);

        let (start, end) = if self.reverse {
            (
                &self.animation.end_transform,
                &self.animation.start_transform,
            )
        } else {
            (
                &self.animation.start_transform,
                &self.animation.end_transform,
            )
        };

        start.lerp(&end, progress)
    }

    pub fn get_progress(&self, time: LevelTime) -> f32 {
        if time > self.end_time {
            return 1.0;
        }
        // if time < self.end_time - self.animation.duration
        if time + self.animation.duration < self.end_time {
            return 0.0;
        }

        // (end_time - duration).inverse_lerp(end_time, time)
        // and then shift everything by + duration
        let progress = self.end_time.inverse_lerp(
            &(self.end_time + self.animation.duration),
            time + self.animation.duration,
        );
        progress
    }
}
