use app::plugin::{Plugin, PluginAppAccess};
use bevy_ecs::{
    prelude::Events,
    schedule::{IntoSystemConfig, IntoSystemSetConfig, SystemSet},
};

use crate::{
    events::{KeyboardInput, MouseInput, MouseMovement},
    input_map::{handle_keyboard_input, handle_mouse_input, InputMap},
};

#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
enum InputPluginSet {
    InputEvents,
    UpdateInputMap,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&mut self, app: &mut PluginAppAccess) {
        app.with_resource(InputMap::new())
            .with_set(InputPluginSet::InputEvents.before(InputPluginSet::UpdateInputMap))
            .with_resource(Events::<MouseMovement>::default())
            .with_system(Events::<MouseMovement>::update_system.in_set(InputPluginSet::InputEvents))
            .with_resource(Events::<MouseInput>::default())
            .with_system(Events::<MouseInput>::update_system.in_set(InputPluginSet::InputEvents))
            .with_resource(Events::<KeyboardInput>::default())
            .with_system(Events::<KeyboardInput>::update_system.in_set(InputPluginSet::InputEvents))
            .with_system(handle_keyboard_input.in_set(InputPluginSet::UpdateInputMap))
            .with_system(
                handle_mouse_input
                    .in_set(InputPluginSet::UpdateInputMap)
                    .after(handle_keyboard_input),
            );
    }
}
