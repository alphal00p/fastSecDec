//! Python settings for the shared native resource scopes.

use pyo3::{exceptions::PyValueError, prelude::*};
use std::time::Duration;
use symbolica::utils::resources::{self, ResourceBudget, ResourceLimits};

pub(crate) fn limits(
    max_process_rss_bytes: Option<u64>,
    max_elapsed_seconds: Option<f64>,
) -> PyResult<ResourceLimits> {
    let max_elapsed = max_elapsed_seconds
        .map(Duration::try_from_secs_f64)
        .transpose()
        .map_err(|_| PyValueError::new_err("max_elapsed_seconds must be finite and nonnegative"))?;
    Ok(ResourceLimits {
        max_process_rss_bytes,
        max_elapsed,
    })
}

pub(crate) fn run<T>(
    py: Python<'_>,
    limits: ResourceLimits,
    operation: impl FnOnce() -> PyResult<T>,
) -> PyResult<T> {
    ResourceBudget::new(limits).run(|| {
        resources::checkpoint().map_err(|error| crate::error::native(py, "generation", error))?;
        let result = operation()?;
        resources::checkpoint().map_err(|error| crate::error::native(py, "generation", error))?;
        Ok(result)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_and_invalid_settings_keep_native_resource_semantics() {
        assert_eq!(limits(None, None).unwrap(), ResourceLimits::default());
        assert_eq!(
            limits(None, Some(0.0)).unwrap().max_elapsed,
            Some(Duration::ZERO)
        );
        for value in [f64::NAN, f64::INFINITY, -1.0] {
            assert!(limits(None, Some(value)).is_err());
        }
    }
}
