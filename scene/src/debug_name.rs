use bevy_ecs::prelude::Component;

#[derive(Component, Clone)]
pub struct DebugName(pub String);
