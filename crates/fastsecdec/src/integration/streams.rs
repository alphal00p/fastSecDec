use crate::integration::{IntegrationError, Result};
use numerica::numerical_integration::MonteCarloRng;
use serde::{Deserialize, Serialize};

/// Native xoshiro256** has period 2^256-1; jump advances by 2^128 draws.
/// At most u64::MAX reservations, each using <= u64::MAX draws, occupy less
/// than 2^193 of that period. No worker can allocate or reset this frontier.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Streams {
    pub next: u64,
    pub state: [u8; 32],
}
impl Streams {
    pub fn new(seed: u64) -> Self {
        Self {
            next: 0,
            state: MonteCarloRng::new(seed, 0).export(),
        }
    }
    pub fn reserve(&mut self, draws: u64) -> Result<(u64, [u8; 32])> {
        if draws == 0 || self.state == [0; 32] {
            return Err(IntegrationError::Invalid(
                "invalid native random stream or draw budget".into(),
            ));
        }
        let next = self.next.checked_add(1).ok_or_else(|| {
            IntegrationError::Invalid("native random stream space exhausted".into())
        })?;
        let identity = self.next;
        let state = self.state;
        let mut rng = MonteCarloRng::import(state);
        rng.jump();
        self.state = rng.export();
        self.next = next;
        Ok((identity, state))
    }
}

pub(crate) fn mc_draws(points: u64, dimension: usize) -> Result<u64> {
    // Registry Numerica 3.0.1 ContinuousGrid<f64>::sample draws once per axis,
    // plus at most one uniform-floor selection. There is no rejection loop.
    let draws = u64::try_from(dimension)
        .ok()
        .and_then(|d| d.checked_add(1))
        .and_then(|d| points.checked_mul(d));
    draws.filter(|n| *n > 0).ok_or_else(|| {
        IntegrationError::Invalid("MC reservation exceeds native RNG draw budget".into())
    })
}
