use bevy_ecs::{
    prelude::Component,
    query::{ReadOnlyWorldQuery, WorldQuery},
    system::Query,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
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

trait GetWithLevelId {}

impl<'w, 's, Q: WorldQuery, F: ReadOnlyWorldQuery> GetWithLevelId for Query<'w, 's, Q, F> {}
