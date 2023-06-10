use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::prelude::*;
use std::time::{Duration, Instant};

#[derive(Resource)]
pub struct Time {
    delta: Duration,
    delta_seconds: f64,
    last_update: Instant,
    start_time: Instant,
}

impl Time {
    fn new() -> Time {
        Time {
            delta: Duration::from_secs(0),
            delta_seconds: 0.0,
            last_update: Instant::now(),
            start_time: Instant::now(),
        }
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds as f32
    }

    pub fn update(&mut self) {
        let delta_time = self.last_update.elapsed();
        self.last_update = Instant::now();

        self.delta = delta_time;
        self.delta_seconds = delta_time.as_secs_f64();
    }

    /// Remember to usually use LevelTime instead
    pub fn time_since_startup(&self) -> Duration {
        self.start_time.elapsed()
    }
}

fn update_time(mut time: ResMut<Time>) {
    time.update();
}

pub struct TimePlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TimePluginSet {
    UpdateTime,
}

impl Plugin for TimePlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app.with_resource(Time::new())
            .with_system(update_time.in_set(TimePluginSet::UpdateTime));
    }
}
