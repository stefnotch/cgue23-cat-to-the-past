use animations::animation::PlayingAnimation;
use app::plugin::Plugin;
use bevy_ecs::prelude::IntoSystemConfig;
use bevy_ecs::{
    prelude::{Query, Res},
    query::With,
    system::Local,
};
use game::level_flags::LevelFlags;
use loader::loader::{Door, Platform};
use scene::level::{Level, LevelId};
use time::time_manager::TimeManager;

fn door_system(
    level_flags: Res<LevelFlags>,
    time: Res<TimeManager>,
    mut query: Query<(&mut PlayingAnimation, &Level), With<Door>>,
    mut door_flag_value: Local<bool>,
) {
    let door_should_open =
        level_flags.get(LevelId::new(2), 0) && level_flags.get(LevelId::new(2), 1);
    if door_should_open != *door_flag_value {
        *door_flag_value = door_should_open;
    } else {
        return;
    }

    let mut animation = query
        .iter_mut()
        .find(|(_, level)| level.id.id() == 2)
        .unwrap()
        .0;
    if door_should_open {
        animation.play_forwards(*time.level_time());
    }
}

fn platform_system(
    level_flags: Res<LevelFlags>,
    time: Res<TimeManager>,
    mut query: Query<(&mut PlayingAnimation, &Level), With<Platform>>,
    mut platform_flag_value: Local<bool>,
) {
    let platform_should_lower = level_flags.get(LevelId::new(2), 0);
    if platform_should_lower != *platform_flag_value {
        *platform_flag_value = platform_should_lower;
    } else {
        return;
    }

    let mut animation = query
        .iter_mut()
        .find(|(_, level)| level.id.id() == 2)
        .unwrap()
        .0;
    if platform_should_lower {
        animation.play_forwards(*time.level_time());
    } else if !platform_should_lower {
        animation.play_backwards(*time.level_time());
    }
}

pub struct Level2Plugin;

impl Plugin for Level2Plugin {
    fn build(&mut self, app: &mut app::plugin::PluginAppAccess) {
        app.with_system(door_system)
            .with_system(platform_system.after(door_system));
    }
}
