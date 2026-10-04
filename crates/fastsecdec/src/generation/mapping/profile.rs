//! Opt-in attribution for the actual mapper in ignored unit-test diagnostics.
//! No state, clocks or report type exists in a normal library build.

use crate::parametric::PolynomialFactor;
use serde::Serialize;
use std::{cell::RefCell, collections::BTreeMap, time::Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) enum Stage {
    Substitution,
    CoordinateFaces,
    SupportExtraction,
    SupportTransformation,
    FactorCollection,
    PolynomialRecognition,
    SparseFallback,
    ResidualCertification,
}

#[derive(Clone, Debug, Default, Serialize)]
pub(crate) struct Measurement {
    calls: usize,
    seconds: f64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct FactorProfile {
    chart: usize,
    source_factor: usize,
    role: String,
    factor_bytes: usize,
    support_monomials: Option<usize>,
    stages: BTreeMap<Stage, Measurement>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SupportCacheProfile {
    pub enabled: bool,
    pub requests: usize,
    pub misses: usize,
    pub entries: usize,
    /// Native Atom representation bytes, not allocator capacity.
    pub key_expression_bytes: usize,
    pub support_rows: usize,
    pub integer_entries: usize,
    /// Inline Integer storage only; excludes containers/allocator overhead.
    pub integer_storage_bytes: usize,
}

#[derive(Default, Serialize)]
pub(crate) struct Snapshot {
    pub factors: Vec<FactorProfile>,
    pub support_cache: Option<SupportCacheProfile>,
}

#[derive(Default)]
struct Profile {
    next_chart: usize,
    next_factor: usize,
    snapshot: Snapshot,
}

std::thread_local! {
    static PROFILE: RefCell<Option<Profile>> = const { RefCell::new(None) };
}

pub(crate) fn begin() {
    PROFILE.with(|value| *value.borrow_mut() = Some(Profile::default()));
}

pub(crate) fn take() -> Snapshot {
    PROFILE.with(|value| {
        value
            .borrow_mut()
            .take()
            .map_or_else(Snapshot::default, |p| p.snapshot)
    })
}

pub(crate) fn begin_chart() {
    PROFILE.with(|value| {
        if let Some(profile) = value.borrow_mut().as_mut() {
            profile.next_chart += 1;
            profile.next_factor = 0;
        }
    });
}

pub(crate) fn begin_factor(factor: &PolynomialFactor) {
    PROFILE.with(|value| {
        if let Some(profile) = value.borrow_mut().as_mut() {
            profile.snapshot.factors.push(FactorProfile {
                chart: profile.next_chart - 1,
                source_factor: profile.next_factor,
                role: format!("{:?}", factor.role()),
                factor_bytes: factor.polynomial().as_view().get_byte_size(),
                support_monomials: None,
                stages: BTreeMap::new(),
            });
            profile.next_factor += 1;
        }
    });
}

pub(crate) fn support_size(monomials: usize) {
    PROFILE.with(|value| {
        if let Some(factor) = value
            .borrow_mut()
            .as_mut()
            .and_then(|p| p.snapshot.factors.last_mut())
        {
            factor.support_monomials = Some(monomials);
        }
    });
}

pub(crate) fn measure<T>(stage: Stage, operation: impl FnOnce() -> T) -> T {
    let active = PROFILE.with(|value| value.borrow().is_some());
    if !active {
        return operation();
    }
    let started = Instant::now();
    let result = operation();
    let seconds = started.elapsed().as_secs_f64();
    PROFILE.with(|value| {
        if let Some(factor) = value
            .borrow_mut()
            .as_mut()
            .and_then(|p| p.snapshot.factors.last_mut())
        {
            let measurement = factor.stages.entry(stage).or_default();
            measurement.calls += 1;
            measurement.seconds += seconds;
        }
    });
    result
}

pub(crate) fn record_cache(summary: SupportCacheProfile) {
    PROFILE.with(|value| {
        if let Some(profile) = value.borrow_mut().as_mut() {
            profile.snapshot.support_cache = Some(summary);
        }
    });
}
