//! Presentation adapter only; native events, cancellation and work stay caller-owned.

use std::time::{Duration, Instant};

use fastsecdec::status::{GenerationSnapshot, GenerationStage};
use feynkit_py::MarimoProgress;
use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::status::PyGenerationSnapshot;

pub(crate) struct GenerationProgress<'a> {
    observer: Option<&'a Py<PyAny>>,
    callback: Option<&'a Py<PyAny>>,
    presenter: Option<MarimoProgress>,
    last_display: Option<(GenerationStage, Option<usize>, Instant)>,
    stopped_title: &'static str,
}

impl<'a> GenerationProgress<'a> {
    pub(crate) fn new(
        py: Python<'_>,
        progress: Option<&'a Py<PyAny>>,
        observer: Option<&'a Py<PyAny>>,
        compiling: bool,
    ) -> PyResult<Self> {
        let automatic = progress.is_some_and(|value| {
            value
                .bind(py)
                .extract::<String>()
                .is_ok_and(|s| s == "auto")
        });
        let callback = if automatic { None } else { progress };
        if callback.is_some_and(|callback| !callback.bind(py).is_callable()) {
            return Err(PyTypeError::new_err(
                "progress must be 'auto', callable, or None",
            ));
        }
        // An existing observer may already render every native event. Avoid
        // adding an automatic second display without suppressing that observer.
        let presenter = if automatic && observer.is_none() {
            MarimoProgress::detect(
                py,
                if compiling {
                    "Compiling sector evaluators"
                } else {
                    "Decomposing integral"
                },
                "Starting native work",
            )?
        } else {
            None
        };
        Ok(Self {
            observer,
            callback,
            presenter,
            last_display: None,
            stopped_title: if compiling {
                "Sector compilation stopped"
            } else {
                "Sector decomposition stopped"
            },
        })
    }

    /// Every callable receives the full event stream. Only automatic widget
    /// painting is coalesced; observer runs first and cancellation short-circuits.
    pub(crate) fn observe(
        &mut self,
        py: Python<'_>,
        status: &GenerationSnapshot,
    ) -> PyResult<bool> {
        py.check_signals()?;
        for callback in [self.observer, self.callback].into_iter().flatten() {
            let value = callback.call1(
                py,
                (PyGenerationSnapshot {
                    inner: status.clone(),
                },),
            )?;
            if !value.is_none(py) && !value.extract::<bool>(py)? {
                return Ok(false);
            }
        }
        if let Some(presenter) = &mut self.presenter {
            let refresh = self.last_display.is_none_or(|(stage, total, when)| {
                stage != status.stage
                    || total != status.total
                    || when.elapsed() >= Duration::from_millis(150)
                    || status.total == Some(status.completed)
            });
            if refresh {
                let title = match status.stage {
                    GenerationStage::Input => "Reading native input",
                    GenerationStage::Parametrization => "Parametrizing integral",
                    GenerationStage::Geometry => "Decomposing sectors",
                    GenerationStage::Mapping => "Mapping sectors",
                    GenerationStage::Symmetry => "Reducing sector symmetries",
                    GenerationStage::Subtraction => "Subtracting endpoints",
                    GenerationStage::Expansion => "Expanding Laurent coefficients",
                    GenerationStage::CoefficientExpansion => "Extracting Laurent coefficients",
                    GenerationStage::Compilation => "Compiling sector evaluators",
                    GenerationStage::Complete => "Sector evaluators ready",
                };
                let subtitle = format!(
                    "{} sectors · {} kernels · {:.1} s · {}",
                    status.sectors, status.kernels, status.elapsed_seconds, status.detail,
                );
                presenter.update(py, title, &subtitle, status.completed, status.total)?;
                self.last_display = Some((status.stage, status.total, Instant::now()));
            }
        }
        Ok(true)
    }

    pub(crate) fn finish<T>(mut self, py: Python<'_>, result: PyResult<T>) -> PyResult<T> {
        if let Some(presenter) = &mut self.presenter {
            let cleanup = presenter.finish(py, result.as_ref().err(), self.stopped_title);
            // The shared presenter also preserves the original exception. Keep
            // the invariant explicit at the scientific binding boundary.
            if result.is_ok() {
                cleanup?;
            }
        }
        result
    }
}
