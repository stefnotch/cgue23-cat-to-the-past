use bevy_ecs::prelude::*;

#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AppStage {
    StartFrame,
    EventUpdate,
    Update,
    UpdatePhysics,
    /// after physics
    BeforeRender,
    Render,
    EndFrame,
}
