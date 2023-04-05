use std::{
    ops::{Add, AddAssign},
    time::Duration,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LevelTime {
    elapsed: Duration,
}
impl LevelTime {
    pub fn zero() -> LevelTime {
        LevelTime {
            elapsed: Duration::from_secs(0),
        }
    }

    pub fn as_secs_f32(&self) -> f32 {
        self.elapsed.as_secs_f32()
    }
}

impl Add<Duration> for LevelTime {
    type Output = Self;

    fn add(self, other: Duration) -> LevelTime {
        LevelTime {
            elapsed: self.elapsed + other,
        }
    }
}

impl AddAssign<Duration> for LevelTime {
    fn add_assign(&mut self, other: Duration) {
        self.elapsed += other;
    }
}
