use app::plugin::Plugin;
use bevy_ecs::{
    prelude::{EventWriter, Events},
    schedule::IntoSystemConfig,
    system::ResMut,
};
use current_level::{CurrentLevel, NextLevel};

pub mod current_level;
pub mod level_id;

pub struct LevelsPlugin;

fn update_current_level(
    mut current_level: ResMut<CurrentLevel>,
    mut event_next_level: EventWriter<NextLevel>,
) {
    if let Some(level_id) = current_level.take_start_next_level() {
        let old_level_id = current_level.level_id;
        current_level.level_id = level_id;
        event_next_level.send(NextLevel {
            level_id,
            old_level_id,
        });
    }
}

impl Plugin for LevelsPlugin {
    fn build(&mut self, app: &mut app::plugin::PluginAppAccess) {
        app //
            .with_resource(CurrentLevel::new())
            .with_resource(Events::<NextLevel>::default())
            .with_system(update_current_level)
            .with_system(Events::<NextLevel>::update_system.after(update_current_level));
    }
}
