use animations::animation::PlayingAnimation;
use app::plugin::Plugin;
use bevy_ecs::{
    prelude::{Query, Res},
    query::With,
    system::Local,
};
use game::level_flags::LevelFlags;
use loader::loader::Door;
use scene::level::{Level, LevelId};
use time::time_manager::TimeManager;

fn door_system(
    level_flags: Res<LevelFlags>,
    time: Res<TimeManager>,
    mut query: Query<(&mut PlayingAnimation, &Level), With<Door>>,
    mut door_flag_value: Local<bool>,
) {
    let door_should_open = level_flags.get(LevelId::new(0), 0);
    if door_should_open != *door_flag_value {
        *door_flag_value = door_should_open;
    } else {
        return;
    }

    let mut animation = query
        .iter_mut()
        .find(|(_, level)| level.id.id() == 0)
        .unwrap()
        .0;
    if door_should_open {
        animation.play_forwards(*time.level_time());
    }
}

pub struct Level0Plugin;

impl Plugin for Level0Plugin {
    fn build(&mut self, app: &mut app::plugin::PluginAppAccess) {
        app.with_system(door_system);
    }
}
