use bevy_ecs::prelude::Component;

pub mod level_flags;

#[derive(Component)]
pub struct Level {
    pub id: LevelId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LevelId {
    id: u32,
}

impl LevelId {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}
