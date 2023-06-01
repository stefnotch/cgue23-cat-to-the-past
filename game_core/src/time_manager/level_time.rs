use std::{
    ops::{Add, AddAssign, Sub},
    time::Duration,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let a = self.elapsed.as_secs_f64();
        let b = other.elapsed.as_secs_f64();
        LevelTime {
            elapsed: Duration::from_secs_f64(a + (b - a) * (t as f64)),
        }
    }

    pub fn inverse_lerp(&self, other: &Self, value: LevelTime) -> f32 {
        let a = self.elapsed.as_secs_f64();
        let b = other.elapsed.as_secs_f64();
        let v = value.elapsed.as_secs_f64();
        ((v - a) / (b - a)) as f32
    }

    pub fn sub_or_zero(&self, delta: Duration) -> LevelTime {
        if self.elapsed > delta {
            LevelTime {
                elapsed: self.elapsed - delta,
            }
        } else {
            LevelTime::zero()
        }
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

impl Add<Duration> for &LevelTime {
    type Output = LevelTime;

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

impl Sub<LevelTime> for LevelTime {
    type Output = Duration;

    fn sub(self, other: LevelTime) -> Duration {
        self.elapsed - other.elapsed
    }
}
