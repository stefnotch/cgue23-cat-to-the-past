use bevy_ecs::prelude::*;
use std::time::Instant;

#[derive(Resource)]
pub struct Time {
    delta_seconds: f64,
    last_update: Instant,
    level_time: Instant,
}

impl Time {
    pub fn new() -> Time {
        Time {
            delta_seconds: 0.0,
            last_update: Instant::now(),
            level_time: Instant::now(),
        }
    }

    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds as f32
    }

    pub fn level_time_seconds(&self) -> f32 {
        self.level_time.elapsed().as_secs_f32()
    }

    pub fn next_level(&mut self) {
        self.level_time = Instant::now();
    }

    pub fn update(&mut self) {
        let delta_time = self.last_update.elapsed().as_secs_f64();
        self.last_update = Instant::now();

        self.delta_seconds = delta_time;
    }
}
