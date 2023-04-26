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

    fn system_set() -> PluginSet {
        PluginSet(TypeId::of::<Self>())
    }
}

/// Every plugin has at least one system set, otherwise the systems of that plugin could become poor homeless orphans.
/// See https://github.com/alice-i-cecile/rfcs/blob/apps-own-scheduling/rfcs/33-apps_own_scheduling.md#why-do-we-need-to-automatically-assign-a-shared-label-to-systems-added-by-a-plugin
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginSet(TypeId);

pub struct PluginAppAccess<'app> {
    app: &'app mut App,
    label: PluginSet,
}

impl<'app> PluginAppAccess<'app> {
    pub fn new<T: Plugin>(app: &'app mut App) -> Self {
        Self {
            app,
            label: T::system_set(),
        }
    }

    pub fn with_resource<T>(&mut self, resource: T) -> &mut Self
    where
        T: Resource,
    {
        self.app.world.insert_resource(resource);
        self
    }

    /// call this with system.in_set(AppStage::...)
    pub fn with_system<Params>(&mut self, system: impl IntoSystemConfig<Params>) -> &mut Self {
        self.app
            .schedule
            .add_system(system.in_set(self.label.clone()));
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
}
