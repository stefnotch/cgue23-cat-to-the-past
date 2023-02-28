use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct Time {
    pub delta_seconds: f64,
}

impl Time {
    pub fn new() -> Time {
        Time { delta_seconds: 0.0 }
    }
}
