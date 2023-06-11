use std::time::{Duration, Instant};

use app::plugin::Plugin;
use bevy_ecs::system::{Res, ResMut, Resource};

use crate::rewind_power::RewindPower;

#[derive(Resource)]
pub struct GameOver {
    is_game_over: bool,
    respawn_start_time: Instant,
}

impl GameOver {
    pub fn new() -> Self {
        Self {
            is_game_over: false,
            respawn_start_time: Instant::now(),
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.is_game_over
    }

    fn set_game_over(&mut self) {
        if self.is_game_over {
            return;
        }
        self.is_game_over = true;
        self.respawn_start_time = Instant::now() + Duration::from_secs(2);
    }
}

fn update_game_over(mut game_over: ResMut<GameOver>, rewind_power: Res<RewindPower>) {
    if game_over.is_game_over() {
        if Instant::now() <= game_over.respawn_start_time {
            // wait for respawn
            // TODO: Freeze player
        } else {
            // start respawn
        }
    } else if rewind_power.is_empty() {
        game_over.set_game_over();
    }
}

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&mut self, app: &mut app::plugin::PluginAppAccess) {
        app //
            .with_resource(GameOver::new())
            .with_system(update_game_over);
    }
}
