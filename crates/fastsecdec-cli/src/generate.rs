use std::{ops::ControlFlow, path::Path, time::Instant};

use fastsecdec::{
    generation::{self, GenerationOptions, GenerationProgress},
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
) -> CliResult<(Artifact, KernelSet)> {
    let started = Instant::now();
    let mut status = GenerationSnapshot {
        stage: GenerationStage::Input,
        completed: 0,
        total: None,
        sectors: 0,
        kernels: 0,
        elapsed_seconds: 0.0,
        detail: format!("Reading {}", path.display()),
    };
    dashboard.generation(&status)?;
    let loaded = input::load(path)?;
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
                status.stage = GenerationStage::Geometry;
                status.completed = *sector;
                status.total = Some(*total);
                status.detail = "Substituting exact sector maps".into();
            }
            GenerationProgress::Subtraction { sector, terms } => {
                status.stage = GenerationStage::Subtraction;
                status.completed = *sector + 1;
                status.detail = format!("{} endpoint subtraction terms", terms);
            }
            GenerationProgress::LaurentExpansion { sector, total } => {
                status.stage = GenerationStage::Expansion;
                status.completed = *sector + 1;
                status.total = Some(*total);
                status.detail = format!("Expanding through ε^{}", options.max_order);
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
    let kernels = generated?.compile()?;
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
    let artifact = Artifact::new(&kernels, provenance)?;
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
