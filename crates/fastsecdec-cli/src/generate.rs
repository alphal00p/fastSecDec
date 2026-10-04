use std::{ops::ControlFlow, path::Path, time::Instant};

use fastsecdec::{
    generation::{self, GenerationOptions, GenerationPhase, GenerationProgress},
    kernel::KernelSet,
    status::{GenerationSnapshot, GenerationStage},
};

use crate::{
    CliResult,
    artifact::{Artifact, Provenance, dependencies},
    display::Dashboard,
    input,
};

pub fn generate(
    path: &Path,
    output: &Path,
    dashboard: &mut Dashboard,
    reference: Option<&crate::reference::PreparedReference>,
) -> CliResult<(Artifact, KernelSet)> {
    let started = Instant::now();
    let mut status = GenerationSnapshot {
        stage: GenerationStage::Input,
        completed: 0,
        total: None,
        sectors: 0,
        kernels: 0,
        elapsed_seconds: 0.0,
        timings: Default::default(),
        detail: format!("Reading {}", path.display()),
    };
    dashboard.generation(&status)?;
    let loaded = input::load(path)?;
    status.timings.input_seconds = loaded.input_seconds;
    status.timings.parametrization_seconds = loaded.parametrization_seconds;
    status.stage = GenerationStage::Parametrization;
    status.completed = 1;
    status.total = Some(1);
    status.detail = format!(
        "{} · {} parameters · {} terms",
        loaded.label,
        loaded.integrand.parameters().len(),
        loaded.integrand.terms().len()
    );
    status.elapsed_seconds = started.elapsed().as_secs_f64();
    dashboard.generation(&status)?;
    let mut options = GenerationOptions {
        max_order: loaded.card.generation.order,
        assume_no_threshold: loaded.card.generation.assume_no_threshold,
        ..GenerationOptions::default()
    };
    options.decomposition.max_sectors = loaded.card.generation.max_sectors;
    options.decomposition.max_support_pairs = loaded.card.generation.max_support_pairs;
    let mut display_error = None;
    let generated = generation::generate(&loaded.integrand, &options, |progress| {
        match progress {
            GenerationProgress::Decomposition(progress) => {
                status.stage = GenerationStage::Geometry;
                status.completed = progress.completed_constraints;
                status.total = Some(progress.total_constraints);
                status.sectors = progress.sectors;
                status.detail = format!(
                    "{:?} · chart {} · {} rays",
                    progress.phase, progress.chart, progress.rays
                );
            }
            GenerationProgress::Factorization { sector, total } => {
                status.stage = GenerationStage::Mapping;
                status.completed = *sector;
                status.total = Some(*total);
                status.detail = "Substituting exact sector maps".into();
            }
            GenerationProgress::Subtraction {
                sector,
                total,
                terms,
            } => {
                status.stage = GenerationStage::Subtraction;
                status.completed = *sector + 1;
                status.total = Some(*total);
                status.detail = format!("{} endpoint subtraction terms", terms);
            }
            GenerationProgress::LaurentExpansion { sector, total } => {
                status.stage = GenerationStage::Expansion;
                status.completed = *sector + 1;
                status.total = Some(*total);
                status.detail = format!("Expanding through ε^{}", options.max_order);
            }
            GenerationProgress::PhaseTiming { phase, seconds } => {
                if *phase == GenerationPhase::Symmetry {
                    status.stage = GenerationStage::Symmetry;
                    status.detail = "Verifying complete density permutations".into();
                }
                let elapsed = match phase {
                    GenerationPhase::Domain => &mut status.timings.domain_seconds,
                    GenerationPhase::Geometry => &mut status.timings.geometry_seconds,
                    GenerationPhase::Mapping => &mut status.timings.mapping_seconds,
                    GenerationPhase::Symmetry => &mut status.timings.symmetry_seconds,
                    GenerationPhase::Subtraction => &mut status.timings.subtraction_seconds,
                    GenerationPhase::Laurent => &mut status.timings.laurent_seconds,
                };
                *elapsed += seconds;
            }
            GenerationProgress::Complete { sectors, orders } => {
                status.stage = GenerationStage::Compilation;
                status.sectors = *sectors;
                status.completed = 0;
                status.total = Some(*sectors);
                status.detail = format!("Portable SymJIT O2 · orders {orders:?}");
            }
        }
        status.elapsed_seconds = started.elapsed().as_secs_f64();
        if let Err(error) = dashboard.generation(&status) {
            display_error = Some(error.to_string());
            return ControlFlow::Break(());
        }
        if dashboard.cancelled() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });
    if let Some(error) = display_error {
        return Err(error.into());
    }
    let generated = generated?;
    let compilation_started = Instant::now();
    let kernels = generated.compile_with_progress(|progress| {
        status.stage = GenerationStage::Compilation;
        status.completed = progress.completed;
        status.total = Some(progress.total);
        status.kernels = progress.completed;
        status.timings.compilation_seconds = progress.elapsed_seconds;
        status.elapsed_seconds = started.elapsed().as_secs_f64();
        status.detail = "Compiling portable SymJIT O2 kernels".into();
        if let Err(error) = dashboard.generation(&status) {
            display_error = Some(error.to_string());
            return ControlFlow::Break(());
        }
        if dashboard.cancelled() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });
    if let Some(error) = display_error {
        return Err(error.into());
    }
    let kernels = kernels?;
    if let Some(reference) = reference {
        reference.validate_identity(kernels.content_id())?;
    }
    status.timings.compilation_seconds = compilation_started.elapsed().as_secs_f64();
    let provenance = Provenance {
        name: loaded.label,
        sources: loaded.sources,
        dependencies: dependencies(),
        domain: format!("{:?}", loaded.integrand.domain()),
        assume_no_threshold: options.assume_no_threshold,
        dimension: loaded.card.integral.dimension,
        regulator: loaded.card.integral.regulator,
        measure: if loaded.loops.is_some() {
            "prod_l d^D k_l / (i*pi^(D/2)); propagators q^2-m^2+i0; no implicit scale factors"
        } else {
            "user-supplied direct density with the declared domain measure"
        }
        .into(),
        measure_multiplier: loaded.card.integral.measure_multiplier,
        max_order: options.max_order,
        integration: serde_json::to_value(loaded.card.integration)?,
    };
    let mut artifact = Artifact::new(&kernels, provenance)?;
    artifact.reference = reference.map(|value| value.settings.clone());
    status.timings.total_seconds = started.elapsed().as_secs_f64();
    artifact.generation_timings = Some(status.timings.clone());
    artifact.save(output)?;
    status.stage = GenerationStage::Complete;
    status.kernels = kernels.sectors().len();
    status.completed = status.kernels;
    status.total = Some(status.kernels);
    status.elapsed_seconds = started.elapsed().as_secs_f64();
    status.detail = format!("Saved {}", output.display());
    dashboard.generation(&status)?;
    Ok((artifact, kernels))
}
