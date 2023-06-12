use app::plugin::Plugin;
use bevy_ecs::{
    prelude::{EventWriter, Events},
    schedule::IntoSystemConfig,
    system::ResMut,
};
use current_level::{CurrentLevel, NextLevel, ResetLevel};

pub mod current_level;
pub mod level_id;

pub struct LevelsPlugin;

fn update_current_level(
    mut current_level: ResMut<CurrentLevel>,
    mut event_next_level: EventWriter<NextLevel>,
    mut event_reset_level: EventWriter<ResetLevel>,
) {
    if let Some(next_level) = current_level.try_start_next_level() {
        let level_id = next_level.level_id;
        event_next_level.send(next_level);
        event_reset_level.send(ResetLevel { level_id });
    }
}

impl Plugin for LevelsPlugin {
    fn build(&mut self, app: &mut app::plugin::PluginAppAccess) {
        app //
            .with_resource(CurrentLevel::new())
            .with_system(update_current_level)
            .with_resource(Events::<NextLevel>::default())
            .with_system(Events::<NextLevel>::update_system.after(update_current_level))
            .with_resource(Events::<ResetLevel>::default())
            .with_system(
                Events::<ResetLevel>::update_system
                    .after(update_current_level)
                    .after(Events::<NextLevel>::update_system),
            );
    }
}
