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

    fn get_progress(&self, time: LevelTime) -> f32 {
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

    pub fn play_forwards(&mut self, time: &LevelTime) {
        self.reverse = false;
        self.end_time = time + self.animation.duration;
    }

    pub fn play_backwards(&mut self, time: &LevelTime) {
        self.reverse = true;
        self.end_time = time + self.animation.duration;
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
