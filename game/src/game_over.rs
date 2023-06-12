use std::time::{Duration, Instant};

use app::plugin::Plugin;
use bevy_ecs::{
    prelude::EventWriter,
    query::{With, Without},
    system::{Query, Res, ResMut, Resource},
};
use levels::{
    current_level::{CurrentLevel, ResetLevel},
    level_id::LevelId,
};
use scene::{level::Spawnpoint, transform::Transform};
use time::time_manager::TimeManager;

use crate::{player::Player, rewind_power::RewindPower};

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

fn update_game_over(
    mut game_over: ResMut<GameOver>,
    rewind_power: Res<RewindPower>,
    time_manager: Res<TimeManager>,
    current_level: Res<CurrentLevel>,
    mut event_reset_level: EventWriter<ResetLevel>,
    // Player spawnpoint resetting
    mut players_query: Query<&mut Transform, With<Player>>,
    spawnpoints: Query<(&Transform, &LevelId), (With<Spawnpoint>, Without<Player>)>,
) {
    if game_over.is_game_over() {
        if Instant::now() <= game_over.respawn_start_time {
            // wait for respawn
        } else if time_manager.level_time().as_secs_f32() > 0.0 {
            // rewind time
            time_manager.rewind_next_frame(4.0);
        } else {
            // respawn
            game_over.is_game_over = false;
            event_reset_level.send(ResetLevel {
                level_id: current_level.level_id,
            });
            for mut transform in players_query.iter_mut() {
                let spawnpoint = spawnpoints
                    .iter()
                    .find(|(_, level_id)| level_id == &&current_level.level_id)
                    .unwrap()
                    .0;
                transform.position = spawnpoint.position;
            }
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
