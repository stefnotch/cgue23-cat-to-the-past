#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LevelId {
    id: u32,
}

impl LevelId {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

// For now the level flags are purely number based. (No enums yet)
pub type FlagId = usize;
