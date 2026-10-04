//! Test-only native-template capture. Skipped series never produce an integral:
//! the selected expansion is exported and generation returns `Cancelled`.

mod replay;
mod tests;

use super::GenerationError;
use std::{cell::RefCell, collections::BTreeMap, fs::File, path::PathBuf, time::Instant};
use symbolica::atom::{Atom, AtomCore, Symbol};

struct Capture {
    target: usize,
    output: PathBuf,
    current: Option<(usize, usize, usize)>,
    started: Instant,
    captured: bool,
}

thread_local! {
    static CAPTURE: RefCell<Option<Capture>> = const { RefCell::new(None) };
}

struct Guard;

impl Guard {
    fn new(target: usize, output: PathBuf) -> Self {
        CAPTURE.with_borrow_mut(|slot| {
            assert!(slot.is_none(), "capture probes cannot nest");
            *slot = Some(Capture {
                target,
                output,
                current: None,
                started: Instant::now(),
                captured: false,
            });
        });
        Self
    }

    fn captured(&self) -> bool {
        CAPTURE.with_borrow(|slot| slot.as_ref().unwrap().captured)
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        CAPTURE.with_borrow_mut(|slot| *slot = None);
    }
}

pub(crate) fn context(index: usize, source_chart: usize, multiplicity: usize) {
    CAPTURE.with_borrow_mut(|slot| {
        if let Some(state) = slot {
            state.current = Some((index, source_chart, multiplicity));
        }
    });
}

pub(super) fn skip_current() -> bool {
    CAPTURE.with_borrow(|slot| {
        slot.as_ref().is_some_and(|state| {
            state
                .current
                .is_some_and(|(index, _, _)| index < state.target)
        })
    })
}

pub(crate) fn reject_uncaptured_result() -> Result<(), GenerationError> {
    // This also covers a missing target or an input with no representatives.
    // An active capture can never return any generated result.
    CAPTURE.with_borrow(|slot| {
        if slot.is_some() {
            Err(GenerationError::Cancelled)
        } else {
            Ok(())
        }
    })
}

pub(super) fn capture(
    expression: &Atom,
    template: &Atom,
    images: &BTreeMap<Symbol, Atom>,
    parameters: &[Symbol],
    regulator: Symbol,
    max_order: i32,
    template_seconds: f64,
) -> Result<(), GenerationError> {
    CAPTURE.with_borrow_mut(|slot| {
        let Some(state) = slot else { return Ok(()) };
        let (index, source_chart, multiplicity) = state.current.expect("capture context");
        assert_eq!(index, state.target, "target must abort generation");
        std::fs::create_dir_all(&state.output).unwrap();
        for (name, value) in [
            ("expression", expression.clone()),
            ("template", template.clone()),
            ("regulator", Atom::var(regulator)),
        ] {
            export(&state.output.join(format!("{name}.atom")), &value);
        }
        let images = images.iter().enumerate().map(|(i, (symbol, value))| {
            export(&state.output.join(format!("image-{i}.atom")), value);
            export(&state.output.join(format!("image-symbol-{i}.atom")), &Atom::var(*symbol));
            serde_json::json!({
                "symbol": Atom::var(*symbol).to_canonical_string(),
                "file": format!("image-{i}.atom"),
                "symbol_file": format!("image-symbol-{i}.atom"),
                "bytes": value.as_view().get_byte_size(),
            })
        }).collect::<Vec<_>>();
        let report = serde_json::json!({
            "representative_index": index, "displayed_ordinal": index + 1,
            "source_chart_index": source_chart, "multiplicity": multiplicity,
            "max_order": max_order,
            "parameters": parameters.iter().map(|p| Atom::var(*p).to_canonical_string()).collect::<Vec<_>>(),
            "regulator": Atom::var(regulator).to_canonical_string(),
            "expression_bytes": expression.as_view().get_byte_size(),
            "template_bytes": template.as_view().get_byte_size(),
            "template_preparation_seconds": template_seconds,
            "images": images,
            "capture_seconds": state.started.elapsed().as_secs_f64(),
            "semantics": "test-only: prior Laurent expansions skipped; selected native template exported; generation returns Cancelled; no usable partial integral",
        });
        std::fs::write(state.output.join("capture.json"), serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        state.captured = true;
        Err(GenerationError::Cancelled)
    })
}

fn export(path: &std::path::Path, expression: &Atom) {
    expression.export(&mut File::create(path).unwrap()).unwrap();
}
