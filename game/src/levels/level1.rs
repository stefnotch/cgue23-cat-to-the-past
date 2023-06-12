use animations::animation::PlayingAnimation;
use app::plugin::Plugin;
use bevy_ecs::{
    prelude::{Query, Res},
    query::With,
    system::Local,
};
use game::level_flags::LevelFlags;
use levels::level_id::LevelId;
use loader::loader::Door;
use time::time_manager::TimeManager;

fn door_system(
    level_flags: Res<LevelFlags>,
    time: Res<TimeManager>,
    mut query: Query<(&mut PlayingAnimation, &LevelId), With<Door>>,
    mut door_flag_value: Local<bool>,
) {
    let level_id = LevelId::new(1);

    let door_should_open = level_flags.get(level_id, 0) && level_flags.get(level_id, 1);
    if door_should_open != *door_flag_value {
        *door_flag_value = door_should_open;
    } else {
        return;
    }

    let mut animation = query
        .iter_mut()
        .find(|(_, level)| level == &&level_id)
        .unwrap()
        .0;
    if door_should_open {
        animation.play_forwards(*time.level_time());
    } else if !door_should_open {
        animation.play_backwards(*time.level_time());
    }
}

pub struct Level1Plugin;

impl Plugin for Level1Plugin {
    fn build(&mut self, app: &mut app::plugin::PluginAppAccess) {
        app.with_system(door_system);
    }
}
