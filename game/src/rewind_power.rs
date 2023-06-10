use app::plugin::Plugin;
use bevy_ecs::system::{Res, ResMut, Resource};
use time::time_manager::TimeManager;

// TODO: Deal with next level (in the game logic)
#[derive(Resource)]
pub struct RewindPower {
    remaining_seconds: f32,
    pub max_seconds: f32,
}

impl RewindPower {
    pub fn new() -> Self {
        Self {
            remaining_seconds: 0.0,
            max_seconds: 1.0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.remaining_seconds <= 0.0
    }

    pub fn get_percent(&self) -> f32 {
        if self.max_seconds == 0.0 {
            return 0.0;
        }
        if self.is_empty() {
            return 0.0;
        }
        self.remaining_seconds / self.max_seconds
    }

    pub fn set_rewind_power(&mut self, rewind_power: f32) {
        self.remaining_seconds = rewind_power;
        self.max_seconds = rewind_power;
    }
}

fn update_rewind_power(mut rewind_power: ResMut<RewindPower>, time_manager: Res<TimeManager>) {
    let consumed_power = time_manager.level_delta_time();
    if consumed_power.is_negative() {
        rewind_power.remaining_seconds =
            (rewind_power.remaining_seconds - consumed_power.duration().as_secs_f32()).max(0.0);
    }
}

pub struct RewindPowerPlugin;

impl Plugin for RewindPowerPlugin {
    fn build(&mut self, app: &mut app::plugin::PluginAppAccess) {
        app //
            .with_resource(RewindPower::new())
            .with_system(update_rewind_power);
    }
}
