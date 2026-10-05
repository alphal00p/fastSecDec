//! Opt-in attribution for the actual mapper in ignored unit-test diagnostics.
//! No state, clocks or report type exists in a normal library build.

use crate::parametric::PolynomialFactor;
use serde::Serialize;
use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    time::Instant,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) enum Stage {
    Substitution,
    CoordinateFaces,
    RegularMonomial,
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
    term: usize,
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
    next_term: usize,
    next_factor: usize,
    snapshot: Snapshot,
    trace: Option<BufWriter<File>>,
}

std::thread_local! {
    static PROFILE: RefCell<Option<Profile>> = const { RefCell::new(None) };
}

pub(crate) fn begin() {
    PROFILE.with(|value| *value.borrow_mut() = Some(Profile::default()));
}

/// Flushed entry/exit observations survive a bounded diagnostic timeout. This
/// owner exists only in the unit-test build and never prints an expression.
pub(crate) fn begin_trace(path: &Path) {
    begin();
    PROFILE.with(|value| {
        value.borrow_mut().as_mut().unwrap().trace =
            Some(BufWriter::new(File::create_new(path).unwrap()));
    });
}

fn trace(stage: Stage, state: &str, seconds: f64) {
    PROFILE.with(|value| {
        let mut value = value.borrow_mut();
        let Some(profile) = value.as_mut() else {
            return;
        };
        let Some(sink) = profile.trace.as_mut() else {
            return;
        };
        let Some(factor) = profile.snapshot.factors.last() else {
            return;
        };
        serde_json::to_writer(
            &mut *sink,
            &serde_json::json!({"stage":stage,"state":state,"seconds":seconds,"factor":factor}),
        )
        .unwrap();
        writeln!(sink).unwrap();
        sink.flush().unwrap();
    });
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
            profile.next_term = 0;
            profile.next_factor = 0;
        }
    });
}

pub(crate) fn begin_term() {
    PROFILE.with(|value| {
        if let Some(profile) = value.borrow_mut().as_mut() {
            profile.next_term += 1;
        }
    });
}

pub(crate) fn begin_factor(factor: &PolynomialFactor) {
    PROFILE.with(|value| {
        if let Some(profile) = value.borrow_mut().as_mut() {
            profile.snapshot.factors.push(FactorProfile {
                chart: profile.next_chart - 1,
                term: profile.next_term - 1,
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
    trace(stage, "begin", 0.0);
    let started = Instant::now();
    let result = operation();
    let seconds = started.elapsed().as_secs_f64();
    trace(stage, "complete", seconds);
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
