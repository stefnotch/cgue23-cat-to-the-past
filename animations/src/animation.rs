use std::time::Duration;

use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::{
    prelude::Component,
    system::{Query, Res},
};
use game_core::time_manager::{level_time::LevelTime, TimeManager};
use scene::transform::Transform;

pub struct Animation {
    pub start_transform: Transform,
    pub end_transform: Transform,
    pub duration: Duration,
}

/// An entity with a PlayingAnimation should not be time tracked.
#[derive(Component)]
pub struct PlayingAnimation {
    animation: Animation,
    end_time: LevelTime,
    /// Also can be used to keep the animation frozen at the start.
    reverse: bool,
}

impl PlayingAnimation {
    pub fn new_frozen(animation: Animation) -> Self {
        Self {
            animation,
            end_time: LevelTime::zero(),
            reverse: true,
        }
    }

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

        start.lerp(&end, progress as f32)
    }

    fn get_progress(&self, time: LevelTime) -> f64 {
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

    pub fn play_forwards(&mut self, time: LevelTime) {
        let remaining_progress = if self.reverse {
            self.get_progress(time)
        } else {
            1.0 - self.get_progress(time)
        };

        self.reverse = false;
        self.end_time = time + self.animation.duration.mul_f64(remaining_progress);
    }

    pub fn play_backwards(&mut self, time: LevelTime) {
        let remaining_progress = if self.reverse {
            1.0 - self.get_progress(time)
        } else {
            self.get_progress(time)
        };
        self.reverse = true;
        self.end_time = time + self.animation.duration.mul_f64(remaining_progress);
    }
}

pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app.with_system(play_animations);
    }
}

fn play_animations(time: Res<TimeManager>, mut query: Query<(&PlayingAnimation, &mut Transform)>) {
    for (playing_animation, mut transform) in query.iter_mut() {
        *transform = playing_animation.get_transform(*time.level_time());
    }
}
