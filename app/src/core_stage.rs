use bevy_ecs::schedule::SystemSet;

#[derive(SystemSet, Clone, PartialEq, Eq, Hash, Debug)]
pub enum CoreStage {
    StartFrame,

    EndFrame,
}
