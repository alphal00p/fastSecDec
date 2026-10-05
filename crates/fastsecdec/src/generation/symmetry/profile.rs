//! Test-only phase attribution. Flushed entries survive a bounded diagnostic.

use std::{cell::RefCell, fs::File, io::Write, path::Path, time::Instant};

#[derive(serde::Serialize)]
pub(in crate::generation) struct GraphProfile {
    pub density_bytes: usize,
    pub vertices: usize,
    pub edges: usize,
    pub encoding_seconds: f64,
    pub canonization_seconds: f64,
}

struct Profile {
    rows: Vec<GraphProfile>,
    trace: Option<File>,
    started: Instant,
    chart: usize,
}

std::thread_local! {
    static PROFILE: RefCell<Option<Profile>> = const { RefCell::new(None) };
}

pub(in crate::generation) fn begin_profile() {
    PROFILE.with(|state| {
        *state.borrow_mut() = Some(Profile {
            rows: Vec::new(),
            trace: None,
            started: Instant::now(),
            chart: 0,
        });
    });
}

pub(in crate::generation) fn begin_trace(path: &Path) {
    begin_profile();
    PROFILE.with(|state| {
        state.borrow_mut().as_mut().unwrap().trace = Some(File::create_new(path).unwrap());
    });
}

pub(in crate::generation) fn chart(index: usize) {
    PROFILE.with(|state| {
        if let Some(state) = state.borrow_mut().as_mut() {
            state.chart = index;
        }
    });
}

pub(in crate::generation) fn trace(stage: &str, state: &str, details: serde_json::Value) {
    PROFILE.with(|profile| {
        let mut profile = profile.borrow_mut();
        let Some(profile) = profile.as_mut() else {
            return;
        };
        let elapsed = profile.started.elapsed().as_secs_f64();
        let Some(sink) = profile.trace.as_mut() else {
            return;
        };
        serde_json::to_writer(
            &mut *sink,
            &serde_json::json!({
                "chart": profile.chart, "elapsed_seconds": elapsed,
                "stage": stage, "state": state, "details": details,
            }),
        )
        .unwrap();
        writeln!(sink).unwrap();
        sink.flush().unwrap();
    });
}

pub(super) fn completed(row: GraphProfile) {
    trace("Canonization", "end", serde_json::to_value(&row).unwrap());
    PROFILE.with(|state| {
        if let Some(state) = state.borrow_mut().as_mut() {
            state.rows.push(row);
        }
    });
}

pub(in crate::generation) fn take_profile() -> Vec<GraphProfile> {
    PROFILE.with(|state| state.borrow_mut().take().map_or_else(Vec::new, |s| s.rows))
}
