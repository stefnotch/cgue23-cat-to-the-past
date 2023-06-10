use bevy_ecs::prelude::Component;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// Spawnpoint for the player in a level.
#[derive(Component, Clone)]
pub struct Spawnpoint;

/// Component that should trigger NextLevel events.
#[derive(Component, Clone)]
pub struct NextLevelTrigger;
