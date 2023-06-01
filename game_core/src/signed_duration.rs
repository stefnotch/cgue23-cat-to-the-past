use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SignedDuration {
    duration: Duration,
    is_negative: bool,
}

impl SignedDuration {
    pub fn new(duration: Duration, is_negative: bool) -> Self {
        if duration.is_zero() {
            return Default::default();
        }
        Self {
            duration,
            is_negative,
        }
    }
}
