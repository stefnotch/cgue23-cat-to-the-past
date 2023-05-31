use std::any::TypeId;

use bevy_ecs::{
    schedule::{IntoSystemConfig, IntoSystemSetConfig, SystemSet},
    system::Resource,
};

use crate::app::App;

/// Modeled after https://docs.rs/bevy/latest/bevy/app/trait.Plugin.html
/// Together with a hint of https://github.com/alice-i-cecile/rfcs/blob/apps-own-scheduling/rfcs/33-apps_own_scheduling.md
/// Remember to properly expose your own plugin system sets to the outside world.
pub trait Plugin: 'static {
    /// Called once when the plugin is added to an App.
    fn build(&mut self, app: &mut PluginAppAccess);

    fn system_set() -> PluginSet
    where
        Self: Sized,
    {
        PluginSet(TypeId::of::<Self>())
    }
}

/// Every plugin has at least one system set, otherwise the systems of that plugin could become poor homeless orphans.
/// See https://github.com/alice-i-cecile/rfcs/blob/apps-own-scheduling/rfcs/33-apps_own_scheduling.md#why-do-we-need-to-automatically-assign-a-shared-label-to-systems-added-by-a-plugin
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginSet(TypeId);

pub struct PluginAppAccess<'app> {
    app: &'app mut App,
    system_set: PluginSet,
}

impl<'app> PluginAppAccess<'app> {
    pub(super) fn new(app: &'app mut App, system_set: PluginSet) -> Self {
        Self { app, system_set }
    }

    pub fn with_resource<T>(&mut self, resource: T) -> &mut Self
    where
        T: Resource,
    {
        self.app.world.insert_resource(resource);
        self
    }

    pub fn with_non_send_resource<T>(&mut self, resource: T) -> &mut Self
    where
        T: 'static,
    {
        self.app.world.insert_non_send_resource(resource);
        self
    }

    pub fn with_system<Params>(&mut self, system: impl IntoSystemConfig<Params>) -> &mut Self {
        self.app
            .schedule
            .add_system(system.in_set(self.system_set.clone()));
        self
    }

    pub fn with_set(&mut self, set: impl IntoSystemSetConfig) -> &mut Self {
        self.app.schedule.configure_set(set);
        self
    }

    pub fn with_plugin<T>(&mut self, plugin: T) -> &mut Self
    where
        T: Plugin,
    {
        //  self.schedule.configure_set(T::system_set().after(CoreStage::StartFrame).before(CoreStage::EndFrame));
        self.app.with_plugin(plugin);
        self
    }

    /// call this with system.in_set(AppStartupStage::...)
    pub fn with_startup_system<Params>(
        &mut self,
        system: impl IntoSystemConfig<Params>,
    ) -> &mut Self {
        self.app
            .startup_schedule
            .as_mut()
            .unwrap()
            .add_system(system);
        self
    }

    /// TODO: This is a hack to get access to the world. We should probably have a better way to do this.
    pub fn world_hack_access(&mut self) -> &bevy_ecs::world::World {
        &self.app.world
    }
}
