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
