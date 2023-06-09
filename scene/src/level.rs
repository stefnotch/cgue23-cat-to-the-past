use bevy_ecs::prelude::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LevelId {
    id: u32,
}

impl LevelId {
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

// For now the level flags are purely number based. (No enums yet)
pub type FlagId = usize;

#[derive(Component, Clone)]
pub struct Level {
    pub id: LevelId,
}

impl Level {
    pub fn new(id: LevelId) -> Self {
        Self { id }
    }
}
