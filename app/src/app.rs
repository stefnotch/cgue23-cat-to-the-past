use std::collections::VecDeque;

use bevy_ecs::{
    schedule::{
        ExecutorKind, IntoSystemConfig, IntoSystemSetConfig, LogLevel, Schedule,
        ScheduleBuildSettings,
    },
    world::World,
};

use crate::plugin::{Plugin, PluginAppAccess, PluginSet};

struct PluginContainer<T>
where
    T: Plugin,
{
    plugin: Option<T>,
    plugin_set: PluginSet,
}

impl<T> PluginContainer<T>
where
    T: Plugin,
{
    fn new(plugin: T) -> Self {
        Self {
            plugin: Some(plugin),
            plugin_set: T::system_set(),
        }
    }
}

/// See https://stackoverflow.com/questions/48027839/how-to-box-a-trait-that-has-associated-types
/// Effectively throwing away the type info
trait PluginContainerDyn {
    fn build(&mut self, app: &mut App);
}

impl<T> PluginContainerDyn for PluginContainer<T>
where
    T: Plugin,
{
    fn build(&mut self, app: &mut App) {
        let mut plugin = self.plugin.take().unwrap();
        let mut plugin_app_access = PluginAppAccess::new(app, self.plugin_set.clone());
        plugin.build(&mut plugin_app_access);
    }
}

/// See https://docs.rs/bevy/latest/bevy/app/struct.App.html
pub struct App {
    pub world: World,
    pub schedule: Schedule,
    pub(crate) startup_schedule: Option<Schedule>,
    plugins: VecDeque<Box<dyn PluginContainerDyn>>,
}

impl App {
    pub fn new() -> Self {
        let mut schedule = Schedule::default();
        schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        schedule.set_apply_final_buffers(true);
        schedule.set_build_settings(ScheduleBuildSettings {
            ambiguity_detection: LogLevel::Warn,
            ..Default::default()
        });

        let world = World::new();

        let mut startup_schedule = Schedule::default();
        startup_schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        schedule.set_apply_final_buffers(true);
        schedule.set_build_settings(ScheduleBuildSettings {
            ambiguity_detection: LogLevel::Warn,
            ..Default::default()
        });

        Self {
            world,
            schedule,
            startup_schedule: Some(startup_schedule),
            plugins: VecDeque::new(),
        }
    }

    pub fn with_plugin<T>(&mut self, plugin: T) -> &mut Self
    where
        T: Plugin,
    {
        self.plugins
            .push_back(Box::new(PluginContainer::new(plugin)));
        self
    }

    // TODO: Get rid of this
    pub fn with_system<Params>(&mut self, system: impl IntoSystemConfig<Params>) -> &mut Self {
        self.schedule.add_system(system);
        self
    }

    pub fn with_set(&mut self, set: impl IntoSystemSetConfig) -> &mut Self {
        self.schedule.configure_set(set);
        self
    }

    pub fn build_plugins(&mut self) {
        // Plugins might add more plugins during building
        loop {
            let mut plugin_container = match self.plugins.pop_front() {
                Some(v) => v,
                None => break,
            };
            plugin_container.build(self);
        }
    }

    pub fn run_startup(&mut self) {
        if let Some(mut startup_schedule) = self.startup_schedule.take() {
            startup_schedule.run(&mut self.world);
        }
    }
}
