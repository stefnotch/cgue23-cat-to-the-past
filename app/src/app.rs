use bevy_ecs::{
    schedule::{
        ExecutorKind, IntoSystemConfig, IntoSystemSetConfig, IntoSystemSetConfigs, Schedule,
    },
    world::World,
};

use crate::{
    core_stage::CoreStage,
    plugin::{Plugin, PluginAppAccess},
};

/// See https://docs.rs/bevy/latest/bevy/app/struct.App.html
pub struct App {
    pub world: World,
    pub schedule: Schedule,
    pub(crate) startup_schedule: Option<Schedule>,
}

impl App {
    pub fn new() -> Self {
        let mut schedule = Schedule::default();
        schedule.configure_sets((CoreStage::StartFrame, CoreStage::EndFrame).chain());
        schedule.set_executor_kind(ExecutorKind::SingleThreaded);

        let world = World::new();

        let mut startup_schedule = Schedule::default();
        startup_schedule.set_executor_kind(ExecutorKind::SingleThreaded);

        Self {
            world,
            schedule,
            startup_schedule: Some(startup_schedule),
        }
    }

    pub fn with_plugin<T>(&mut self, mut plugin: T) -> &mut Self
    where
        T: Plugin,
    {
        //  self.schedule.configure_set(T::system_set().after(CoreStage::StartFrame).before(CoreStage::EndFrame));
        plugin.build(&mut PluginAppAccess::new::<T>(self));
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

    pub fn run_startup(&mut self) {
        if let Some(mut startup_schedule) = self.startup_schedule.take() {
            startup_schedule.run(&mut self.world);
        }
    }
}
