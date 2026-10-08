//! Caller-owned process residency. Native libraries never spawn these workers.
pub(crate) mod child;
pub(crate) mod protocol;
mod worker;

pub(crate) use worker::{ProcessPool, ProcessWorker};

/// Linux retains the mapped image through procfs even if a concurrent build
/// atomically replaces its pathname. Workers must use this coordinator's build.
pub(crate) fn executable() -> std::io::Result<std::path::PathBuf> {
    #[cfg(target_os = "linux")]
    if std::path::Path::new("/proc/self/exe").exists() {
        return Ok("/proc/self/exe".into());
    }
    std::env::current_exe()
}

#[cfg(test)]
mod sampling_tests;
#[cfg(test)]
mod tests;
