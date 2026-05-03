use std::sync::Arc;

use chrono::{DateTime, Utc};

/// Source of "now" — pluggable for deterministic tests.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[derive(Debug)]
pub struct FixedClock {
    instant: DateTime<Utc>,
}

impl FixedClock {
    #[must_use]
    pub fn new(instant: DateTime<Utc>) -> Self {
        Self { instant }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.instant
    }
}

#[must_use]
pub fn system_clock() -> Arc<dyn Clock> {
    Arc::new(SystemClock)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn fixed_clock_returns_the_same_instant() {
        let t = Utc.with_ymd_and_hms(2026, 5, 3, 12, 0, 0).unwrap();
        let clock = FixedClock::new(t);
        assert_eq!(clock.now(), t);
        assert_eq!(clock.now(), t);
    }

    #[test]
    fn system_clock_returns_recent_instant() {
        let clock = SystemClock;
        let now = clock.now();
        let delta = (Utc::now() - now).num_seconds().abs();
        assert!(
            delta < 5,
            "system clock should be near Utc::now: {delta}s drift"
        );
    }
}
