use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::{
    prelude::Events,
    schedule::{IntoSystemConfig, SystemSet},
};

use crate::{
    events::{KeyboardInput, MouseInput, MouseMovement},
    input_map::{handle_keyboard_input, handle_mouse_input, InputMap},
};

#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub enum InputPluginSet {
    UpdateInput,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app.with_resource(InputMap::new())
            .with_system(handle_keyboard_input.in_set(InputPluginSet::UpdateInput))
            .with_system(handle_mouse_input.in_set(InputPluginSet::UpdateInput))
            .with_resource(Events::<MouseMovement>::default())
            .with_system(Events::<MouseMovement>::update_system.in_set(InputPluginSet::UpdateInput))
            .with_resource(Events::<MouseInput>::default())
            .with_system(Events::<MouseInput>::update_system.in_set(InputPluginSet::UpdateInput))
            .with_resource(Events::<KeyboardInput>::default())
            .with_system(
                Events::<KeyboardInput>::update_system.in_set(InputPluginSet::UpdateInput),
            );
    }
}
