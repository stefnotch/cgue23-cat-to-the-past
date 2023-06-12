use animations::animation::PlayingAnimation;
use app::plugin::Plugin;
use bevy_ecs::{
    prelude::{Query, Res},
    query::With,
    schedule::IntoSystemConfig,
    system::{Local, ResMut},
};
use game::level_flags::{FlagChange, LevelFlags};
use levels::level_id::LevelId;
use loader::loader::Door;
use time::time_manager::game_change::GameChangeHistory;
use time::time_manager::TimeManager;

fn door_system(
    level_flags: Res<LevelFlags>,
    time: Res<TimeManager>,
    mut query: Query<(&mut PlayingAnimation, &LevelId), With<Door>>,
    mut door_flag_value: Local<bool>,
) {
    let level_id = LevelId::new(0);

    let door_should_close = level_flags.get(level_id, 1);
    if door_should_close != *door_flag_value {
        *door_flag_value = door_should_close;
    } else {
        return;
    }

    let mut animation = query
        .iter_mut()
        .find(|(_, level)| level == &&level_id)
        .unwrap()
        .0;
    if door_should_close {
        animation.play_forwards(*time.level_time());
    } else if !door_should_close {
        animation.play_backwards(*time.level_time());
    }
}

fn laser_system(
    mut level_flags: ResMut<LevelFlags>,
    mut game_change_history: ResMut<GameChangeHistory<FlagChange>>,
) {
    let level_id = LevelId::new(0);
    let laser_activated = level_flags.get(level_id, 0);
    if laser_activated {
        level_flags.set_and_record(level_id, 1, true, &mut game_change_history);
    }
}

pub struct Level0Plugin;

impl Plugin for Level0Plugin {
    fn build(&mut self, app: &mut app::plugin::PluginAppAccess) {
        app
            //
            .with_system(laser_system)
            .with_system(door_system.after(laser_system));
    }
}
