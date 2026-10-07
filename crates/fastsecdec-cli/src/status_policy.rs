//! Caller-owned observation cadence. No numerical settings or estimators are
//! changed. Generation polls still reach cancellation; integration asks before
//! constructing an expensive native snapshot.
use std::time::Duration;

use fastsecdec::status::{GenerationSnapshot, GenerationStage};

/// Stage transitions are immediate; ordinary counts are coalesced at the
/// configured cadence, including geometry and representative completions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenerationBoundary {
    stage: GenerationStage,
}

impl From<&GenerationSnapshot> for GenerationBoundary {
    fn from(snapshot: &GenerationSnapshot) -> Self {
        Self {
            stage: snapshot.stage,
        }
    }
}

pub struct StatusCadence {
    interval: Duration,
    last: Option<Duration>,
}

impl StatusCadence {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last: None,
        }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub fn due(&mut self, now: Duration, force: bool) -> bool {
        if force
            || self
                .last
                .is_none_or(|last| now.saturating_sub(last) >= self.interval)
        {
            self.last = Some(now);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn caller_clock_controls_cadence_and_forced_boundaries_always_emit() {
        let ms = Duration::from_millis;
        let mut cadence = StatusCadence::new(ms(100));
        assert!(cadence.due(ms(0), false));
        assert!(!cadence.due(ms(99), false));
        assert!(cadence.due(ms(100), false));
        assert!(cadence.due(ms(101), true));
        assert!(!cadence.due(ms(200), false));
        assert!(cadence.due(ms(201), false));
        // Failure/cancellation/stage/final events cannot be hidden by a deadline.
        for _ in 0..4 {
            assert!(cadence.due(ms(201), true));
        }
        let mut every_batch = StatusCadence::new(Duration::ZERO);
        for _ in 0..4 {
            assert!(every_batch.due(ms(0), false));
        }
    }
}
