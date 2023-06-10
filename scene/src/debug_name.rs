use bevy_ecs::prelude::Component;

#[derive(Component, Clone, Debug)]
pub struct DebugName(pub String);
