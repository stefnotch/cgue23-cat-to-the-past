use bevy_ecs::prelude::Component;

// For now the level flags are purely number based. (No enums yet)
pub type FlagId = usize;

/// Spawnpoint for the player in a level.
#[derive(Component, Clone)]
pub struct Spawnpoint;

/// Component that should trigger NextLevel events.
#[derive(Component, Clone)]
pub struct NextLevelTrigger;
