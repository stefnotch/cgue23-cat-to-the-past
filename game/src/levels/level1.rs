use animations::animation::PlayingAnimation;
use app::plugin::Plugin;
use bevy_ecs::prelude::{Query, Res};
use game::level_flags::LevelFlags;
use loader::loader::Door;
use scene::level::LevelId;
use time::time_manager::TimeManager;

fn door_system(
    level_flags: Res<LevelFlags>,
    time: Res<TimeManager>,
    mut query: Query<(&mut PlayingAnimation, &mut Door)>,
) {
    let door_should_open = level_flags.get(LevelId::new(0), 0);
    let (mut animation, mut door) = query.single_mut();
    if door_should_open && !door.is_open {
        door.is_open = true;
        animation.play_forwards(*time.level_time());
    } else if !door_should_open && door.is_open {
        door.is_open = false;
        animation.play_backwards(*time.level_time());
    }
}

pub struct Level1Plugin;

impl Plugin for Level1Plugin {
    fn build(&mut self, app: &mut app::plugin::PluginAppAccess) {
        app.with_system(door_system);
    }
}
