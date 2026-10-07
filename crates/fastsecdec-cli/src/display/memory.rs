//! CLI-owned, bounded OS sampling. RSS belongs to the process, not its threads.

use std::time::{Duration, Instant};

use sysinfo::{MemoryRefreshKind, Pid, ProcessRefreshKind, ProcessesToUpdate, System};

const SAMPLE_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Default, serde::Serialize)]
pub(super) struct Snapshot {
    /// Resident memory of this process, including every worker thread.
    pub process_rss_bytes: Option<u64>,
    /// Maximum sampled RSS since monitoring began; not the OS high-water mark.
    pub observed_peak_rss_bytes: Option<u64>,
    pub system_used_bytes: Option<u64>,
    pub system_total_bytes: Option<u64>,
    /// Native free RAM. On macOS this excludes speculative pages; it is not
    /// the broader, overlapping active/inactive reclaimable-memory counter.
    pub system_free_bytes: Option<u64>,
    /// Native OS accounting, preserved for JSON consumers. On macOS this
    /// includes active and inactive pages already counted as used memory, so
    /// it is neither free RAM nor total minus used.
    pub system_available_bytes: Option<u64>,
    pub sample_interval_ms: u64,
    /// OS user+system CPU across all process threads, since this invocation.
    pub process_cpu_seconds: Option<f64>,
    pub sample_age_ms: Option<u64>,
}

pub(super) struct Monitor {
    system: Option<System>,
    pid: Option<Pid>,
    sampled_at: Option<Instant>,
    snapshot: Snapshot,
    cpu_baseline_ms: Option<u64>,
    last_cpu_ms: Option<u64>,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            system: sysinfo::IS_SUPPORTED_SYSTEM.then(System::new),
            pid: sysinfo::get_current_pid().ok(),
            sampled_at: None,
            cpu_baseline_ms: None,
            last_cpu_ms: None,
            snapshot: Snapshot {
                sample_interval_ms: SAMPLE_INTERVAL.as_millis() as u64,
                ..Snapshot::default()
            },
        }
    }

    pub fn begin_integration(&mut self) {
        self.refresh();
        self.cpu_baseline_ms = self.last_cpu_ms;
        self.snapshot.process_cpu_seconds = self.cpu_baseline_ms.map(|_| 0.0);
    }

    pub fn finish(&mut self) -> Snapshot {
        self.refresh();
        self.sample()
    }

    /// Called by the dashboard's existing polling loop; no background threads,
    /// process enumeration, shell commands, or work on numerical hot paths.
    pub fn sample(&mut self) -> Snapshot {
        if self
            .sampled_at
            .is_none_or(|at| at.elapsed() >= SAMPLE_INTERVAL)
        {
            self.refresh();
        }
        Snapshot {
            sample_age_ms: self
                .sampled_at
                .map(|at| at.elapsed().as_millis().min(u128::from(u64::MAX)) as u64),
            ..self.snapshot
        }
    }

    fn refresh(&mut self) {
        let Some(system) = &mut self.system else {
            return;
        };
        system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        let total = system.total_memory();
        self.snapshot.system_total_bytes = (total > 0).then_some(total);
        self.snapshot.system_used_bytes = (total > 0).then(|| system.used_memory());
        self.snapshot.system_free_bytes = (total > 0).then(|| system.free_memory());
        self.snapshot.system_available_bytes = (total > 0).then(|| system.available_memory());
        self.snapshot.process_rss_bytes = self.pid.and_then(|pid| {
            let updated = system.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[pid]),
                true,
                ProcessRefreshKind::nothing()
                    .with_memory()
                    .with_cpu()
                    .without_tasks(),
            );
            // A failed current-process query must not appear as a zero reading
            // or retain a stale process entry after an unsuccessful refresh.
            let process = (updated > 0).then(|| system.process(pid)).flatten();
            let cpu = process.map(|process| process.accumulated_cpu_time());
            let monotonic = cpu.filter(|value| self.last_cpu_ms.is_none_or(|last| *value >= last));
            self.snapshot.process_cpu_seconds = self
                .cpu_baseline_ms
                .zip(monotonic)
                .and_then(|(baseline, current)| current.checked_sub(baseline))
                .map(|milliseconds| milliseconds as f64 / 1000.0);
            if monotonic.is_some() {
                self.last_cpu_ms = monotonic;
            }
            process
                .map(|process| process.memory())
                .filter(|&rss| rss > 0)
        });
        if let Some(rss) = self.snapshot.process_rss_bytes {
            self.snapshot.observed_peak_rss_bytes =
                Some(self.snapshot.observed_peak_rss_bytes.unwrap_or(0).max(rss));
        }
        self.sampled_at = Some(Instant::now());
    }
}

impl Snapshot {
    /// macOS's broad VM "available" counter includes used active pages. Prefer
    /// genuinely free pages there; retain native available memory elsewhere.
    /// Keep this label/value pair shared by every dashboard presentation.
    pub fn free_or_available(&self) -> (&'static str, Option<u64>) {
        if cfg!(target_os = "macos") {
            ("Free", self.system_free_bytes)
        } else {
            ("Available", self.system_available_bytes)
        }
    }

    pub fn process_line(&self) -> String {
        format!(
            "Process RSS {} · observed peak {}",
            bytes(self.process_rss_bytes),
            bytes(self.observed_peak_rss_bytes),
        )
    }

    pub fn system_line(&self) -> String {
        let (label, value) = self.free_or_available();
        format!(
            "System RAM · used {} / {} · {label} {} (OS)",
            bytes(self.system_used_bytes),
            bytes(self.system_total_bytes),
            bytes(value),
        )
    }
}

pub(super) fn bytes(value: Option<u64>) -> String {
    let Some(value) = value else {
        return "unavailable".into();
    };
    let (scale, unit) = if value >= 1 << 40 {
        ((1_u64 << 40) as f64, "TiB")
    } else if value >= 1 << 30 {
        ((1_u64 << 30) as f64, "GiB")
    } else {
        ((1_u64 << 20) as f64, "MiB")
    };
    format!("{:.1} {unit}", value as f64 / scale)
}
