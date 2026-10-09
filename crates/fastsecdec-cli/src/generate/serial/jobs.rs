//! File-backed synchronous jobs. This module runs only in recyclable children.
use crate::{
    CliResult,
    artifact::{self, GenerationRecord, Provenance},
    input,
};
use fastsecdec::{
    generation::{self, streaming as native},
    kernel::{
        CompilationSettings, PrecisionPolicy,
        indexed::{self, RecordReceipt},
    },
    status::{GenerationSnapshot, GenerationStage, GenerationTimings},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    ops::ControlFlow,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Job {
    pub root: PathBuf,
    pub response: PathBuf,
    pub request: Request,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum Request {
    Prepare {
        input: PathBuf,
        workers: usize,
        #[serde(default)]
        overrides: crate::config::GenerationOverrides,
    },
    Discover {
        preparation: PathBuf,
        program_recipe: indexed::ProgramRecipe,
        source_id: String,
        dimension: usize,
        index: usize,
    },
    Symmetry {
        preparation: PathBuf,
        chart: native::DiscoveredSector,
        candidates: Vec<native::DiscoveredSector>,
    },
    Formula {
        preparation: PathBuf,
        chart: native::DiscoveredSector,
    },
    Sector {
        job: native::SectorJob,
        max_order: i32,
        evaluator: CompilationSettings,
        output: PathBuf,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Prepared {
    pub native: native::Preparation,
    pub provenance: Provenance,
    pub generation: GenerationRecord,
    pub timings: GenerationTimings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Compiled {
    pub program_recipe: indexed::ProgramRecipe,
    pub source_id: String,
    pub source_index: usize,
    pub data: PathBuf,
    pub receipts: Vec<RecordReceipt>,
    pub source_chart_modes: std::collections::BTreeMap<usize, generation::GenerationMode>,
    pub compilation_seconds: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum Response {
    Prepared(Box<Prepared>),
    Discovered(native::DiscoveredSector),
    Symmetry(native::SymmetryAssignment),
    Formula(native::FormulaRecord),
    Compiled(Compiled),
}

pub(crate) fn read_response(path: &Path) -> CliResult<Response> {
    Ok(serde_json::from_reader(File::open(path)?)?)
}

fn read_prepared(path: &Path) -> CliResult<Prepared> {
    match read_response(path)? {
        Response::Prepared(value) => Ok(*value),
        _ => Err("generation preparation receipt has the wrong job type".into()),
    }
}

pub(crate) fn execute(
    path: &Path,
    emit: &mut dyn FnMut(GenerationSnapshot) -> std::io::Result<()>,
) -> CliResult<()> {
    let job: Job = serde_json::from_reader(File::open(path)?)?;
    fs::create_dir_all(&job.root)?;
    let started = Instant::now();
    let mut snapshot = GenerationSnapshot {
        stage: GenerationStage::Input,
        completed: 0,
        total: None,
        sectors: 0,
        kernels: 0,
        elapsed_seconds: 0.,
        timings: Default::default(),
        coefficient_expansion: None,
        formula_preparation: None,
        detail: String::new(),
    };
    // A native subtraction callback also serves as a cancellation poll and can
    // run far more frequently than a dashboard refresh. Coalesce those updates
    // before serializing so bounded control-channel backpressure cannot turn
    // algebra into a wait for the coordinator to consume identical snapshots.
    // The separate parent watchdog remains active between displayed updates.
    let mut progress_gate = ProgressGate::default();
    // Progress is bounded metadata. It never contains a native sector object.
    let mut observe = |order, progress: &generation::GenerationProgress| {
        snapshot.observe_generation(order, progress);
        let elapsed = started.elapsed();
        snapshot.elapsed_seconds = elapsed.as_secs_f64();
        if !progress_gate.accept(snapshot.stage, elapsed) {
            return ControlFlow::Continue(());
        }
        snapshot.detail = snapshot.detail.chars().take(4096).collect();
        if emit(snapshot.clone()).is_err() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    };
    let response = match job.request {
        Request::Prepare {
            input,
            workers,
            overrides,
        } => {
            let loaded = input::load_observed_with_overrides(&input, overrides, |_| Ok(()))?;
            let settings = &loaded.card.generation;
            let mut options = generation::GenerationOptions {
                max_order: settings.order,
                mode: settings.mode,
                subtraction: settings.subtraction,
                assume_no_threshold: settings.assume_no_threshold,
                program_recipe: settings.program_recipe(),
                coefficient_expansion: settings.coefficient_expansion.clone(),
                ..Default::default()
            };
            options.decomposition.max_sectors = settings.max_sectors;
            options.decomposition.max_support_pairs = settings.max_support_pairs;
            let preparation = native::prepare_with_runtime(
                &loaded.integrand,
                &options,
                &loaded.runtime_parameters,
                &loaded.runtime_mass_constraints,
                &job.root,
                |p| observe(options.max_order, p),
            )?;
            let generation = GenerationRecord {
                workers,
                mode: Some(options.mode),
                subtraction: Some(options.subtraction),
                source_chart_modes: None,
                formula_preparation: None,
                contraction_mode: loaded.loops.map(|_| settings.contraction_mode),
                requested_coefficient_expansion: options.coefficient_expansion.method,
                evaluator: Some(settings.evaluator),
            };
            let provenance = Provenance {
                name: loaded.label, sources: loaded.sources, dependencies: artifact::dependencies(),
                domain: format!("{:?}", loaded.integrand.domain()), assume_no_threshold: options.assume_no_threshold,
                dimension: loaded.card.integral.dimension, regulator: loaded.card.integral.regulator,
                measure: if loaded.loops.is_some() {
                    "prod_l d^D k_l / (i*pi^(D/2)); propagators q^2-m^2+i0; no implicit scale factors"
                } else { "user-supplied direct density with the declared domain measure" }.into(),
                measure_multiplier: loaded.card.integral.measure_multiplier, max_order: options.max_order,
                integration: serde_json::to_value(loaded.card.integration)?,
                family_preparation: loaded.family_preparation,
                model_parameter_defaults: loaded.model_parameter_defaults,
            };
            Response::Prepared(Box::new(Prepared {
                native: preparation,
                provenance,
                generation,
                timings: GenerationTimings {
                    input_seconds: loaded.input_seconds,
                    parametrization_seconds: loaded.parametrization_seconds,
                    ..snapshot.timings
                },
            }))
        }
        Request::Discover {
            preparation,
            program_recipe,
            source_id,
            dimension,
            index,
        } => {
            let prepared = read_prepared(&preparation)?;
            if program_recipe != prepared.native.program_recipe
                || source_id != prepared.native.source.blake3
                || dimension != prepared.native.dimension
            {
                return Err("discovery request differs from its prepared source or recipe".into());
            }
            let map = prepared
                .native
                .charts
                .get(index)
                .ok_or("unknown discovery chart")?;
            Response::Discovered(native::discover(&job.root, &prepared.native, map, |p| {
                observe(prepared.provenance.max_order, p)
            })?)
        }
        Request::Symmetry {
            preparation,
            chart,
            candidates,
        } => {
            let prepared = read_prepared(&preparation)?;
            Response::Symmetry(native::compare_symmetry(
                &job.root,
                &prepared.native,
                &chart,
                &candidates,
                |p| observe(prepared.provenance.max_order, p),
            )?)
        }
        Request::Formula { preparation, chart } => {
            let prepared = read_prepared(&preparation)?;
            Response::Formula(native::build_formula(
                &job.root,
                &prepared.native,
                &chart,
                |p| observe(prepared.provenance.max_order, p),
            )?)
        }
        Request::Sector {
            job: sector,
            max_order,
            evaluator,
            output,
        } => {
            let unit = native::generate_sector(&job.root, &sector, |p| observe(max_order, p))?;
            let modes = unit
                .generated
                .metadata()
                .charts()
                .iter()
                .map(|chart| {
                    let mode = chart
                        .kernel_sector()
                        .map_or(generation::GenerationMode::Symbolic, |id| {
                            unit.generated.sectors()[id].generation_mode()
                        });
                    (unit.source_indices[chart.source_index()], mode)
                })
                .collect();
            let compile_started = Instant::now();
            let kernels = unit
                .generated
                .compile_with_settings_parameters_and_dispatch(
                    PrecisionPolicy::default(),
                    &unit.runtime_parameters,
                    evaluator,
                    &mut |jobs| jobs.map(|job| job.run()).collect(),
                    |progress| {
                        snapshot.observe_compilation(progress);
                        snapshot.elapsed_seconds = started.elapsed().as_secs_f64();
                        if emit(snapshot.clone()).is_err() {
                            ControlFlow::Break(())
                        } else {
                            ControlFlow::Continue(())
                        }
                    },
                )?
                .with_runtime_mass_constraints(unit.runtime_mass_constraints)?;
            let compilation_seconds = compile_started.elapsed().as_secs_f64();
            drop(unit.generated);
            let temporary = output.with_extension("writing");
            let result = (|| -> CliResult<Vec<RecordReceipt>> {
                let mut data = BufWriter::new(File::create(&temporary)?);
                let receipts = indexed::write_unit(&mut data, &kernels, unit.source_indices)?;
                data.flush()?;
                data.get_ref().sync_all()?;
                drop(data);
                fs::rename(&temporary, &output)?;
                File::open(output.parent().ok_or("sector output has no directory")?)?.sync_all()?;
                Ok(receipts)
            })();
            if result.is_err() {
                let _ = fs::remove_file(&temporary);
            }
            Response::Compiled(Compiled {
                program_recipe: sector.program_recipe,
                source_id: sector.source.blake3.clone(),
                source_index: sector.index,
                data: output,
                receipts: result?,
                source_chart_modes: modes,
                compilation_seconds,
            })
        }
    };
    // A receipt is visible only after every heavy output it references is durable.
    artifact::atomic_write(&job.response, &serde_json::to_vec(&response)?)?;
    File::open(job.response.parent().ok_or("receipt has no directory")?)?.sync_all()?;
    Ok(())
}

#[derive(Default)]
struct ProgressGate {
    last: Option<(GenerationStage, Duration)>,
}

impl ProgressGate {
    fn accept(&mut self, stage: GenerationStage, now: Duration) -> bool {
        if let Some((previous_stage, previous_time)) = self.last
            && previous_stage == stage
            && now.saturating_sub(previous_time) < Duration::from_millis(50)
        {
            return false;
        }
        self.last = Some((stage, now));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frequent_native_polls_are_coalesced_but_stage_changes_are_immediate() {
        let mut gate = ProgressGate::default();
        let now = Duration::from_secs(1);
        assert!(gate.accept(GenerationStage::Mapping, now));
        for elapsed in 0..50 {
            assert!(!gate.accept(
                GenerationStage::Mapping,
                now + Duration::from_millis(elapsed)
            ));
        }
        assert!(gate.accept(GenerationStage::Mapping, now + Duration::from_millis(50)));
        assert!(gate.accept(
            GenerationStage::FormulaPreparation,
            now + Duration::from_millis(51)
        ));
    }
}
