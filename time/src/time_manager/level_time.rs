use std::{
    ops::{Add, AddAssign, Sub},
    time::Duration,
};

use crate::signed_duration::SignedDuration;

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

    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        let a = self.elapsed.as_secs_f64();
        let b = other.elapsed.as_secs_f64();
        LevelTime {
            elapsed: Duration::from_secs_f64(a + (b - a) * t),
        }
    }

    pub fn inverse_lerp(&self, other: &Self, value: LevelTime) -> f64 {
        let a = self.elapsed.as_secs_f64();
        let b = other.elapsed.as_secs_f64();
        let v = value.elapsed.as_secs_f64();
        (v - a) / (b - a)
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
    type Output = SignedDuration;

    fn sub(self, other: LevelTime) -> SignedDuration {
        let is_negative = self.elapsed < other.elapsed;
        if is_negative {
            SignedDuration::new(other.elapsed - self.elapsed, is_negative)
        } else {
            SignedDuration::new(self.elapsed - other.elapsed, is_negative)
        }
    }
}
