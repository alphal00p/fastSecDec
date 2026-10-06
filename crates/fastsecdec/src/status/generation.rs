//! Shared snapshot transitions for terminal and HEPKit observers.
use super::{CoefficientExpansionSnapshot, GenerationSnapshot, GenerationStage};
use crate::{
    generation::{
        CoefficientExpansionMethod, CoefficientExpansionStage, GenerationPhase, GenerationProgress,
    },
    kernel::CompilationProgress,
};

impl GenerationSnapshot {
    /// Apply one native generation event without presentation or scheduling.
    ///
    /// Feed each event once: phase timings are increments. The caller owns wall
    /// time, cancellation and update throttling. Generation completion starts
    /// compilation; only the caller can declare its complete delivery successful.
    pub fn observe_generation(&mut self, max_order: i32, progress: &GenerationProgress) {
        let status = self;
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
            GenerationProgress::Symmetry { completed, total } => {
                status.stage = GenerationStage::Symmetry;
                status.completed = *completed;
                status.total = Some(*total);
                status.detail = "Verifying complete density permutations on coordinator".into();
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
                status.detail = format!("Expanding through ε^{}", max_order);
            }
            GenerationProgress::CoefficientExpansion {
                sector,
                total,
                stage,
                attempt,
                relative_width,
                formal_pieces,
                requests,
            } => {
                let effective_method = if *stage == CoefficientExpansionStage::PhysicalFallback {
                    CoefficientExpansionMethod::Physical
                } else {
                    status
                        .coefficient_expansion
                        .as_ref()
                        .filter(|previous| previous.sector == *sector)
                        .map_or(CoefficientExpansionMethod::NativeNamed, |previous| {
                            previous.effective_method
                        })
                };
                status.stage = GenerationStage::CoefficientExpansion;
                // An in-flight representative is not completed work.
                status.completed =
                    *sector + usize::from(*stage == CoefficientExpansionStage::Complete);
                status.total = Some(*total);
                status.coefficient_expansion = Some(CoefficientExpansionSnapshot {
                    sector: *sector,
                    requested_method: CoefficientExpansionMethod::NativeNamed,
                    effective_method,
                    stage: *stage,
                    attempt: *attempt,
                    relative_width: *relative_width,
                    formal_pieces: *formal_pieces,
                    requests: *requests,
                });
                status.detail = if effective_method == CoefficientExpansionMethod::Physical {
                    format!("{} · physical endpoint fallback", activity(*stage))
                } else {
                    format!(
                        "{} · current attempt {} (width {}) · {} formal pieces · {} distinct requests · {} aliases",
                        activity(*stage),
                        attempt,
                        relative_width,
                        formal_pieces,
                        requests.unique_requests,
                        requests.aliases
                    )
                };
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
                    GenerationPhase::CoefficientExpansion => {
                        &mut status.timings.coefficient_expansion_seconds
                    }
                };
                *elapsed += seconds;
            }
            GenerationProgress::Complete { sectors, orders } => {
                status.stage = GenerationStage::Compilation;
                status.sectors = *sectors;
                status.completed = 0;
                status.total = Some(*sectors);
                status.detail = format!("Building sector evaluators · orders {orders:?}");
            }
        }
    }

    /// Apply native compilation progress. The caller still owns elapsed wall
    /// time and any subsequent artifact publication or transport.
    pub fn observe_compilation(&mut self, progress: &CompilationProgress) {
        self.stage = GenerationStage::Compilation;
        self.completed = progress.completed;
        self.total = Some(progress.total);
        self.kernels = progress.completed;
        self.timings.compilation_seconds = progress.elapsed_seconds;
        self.detail = "Building sector evaluators".into();
    }
}

fn activity(stage: CoefficientExpansionStage) -> &'static str {
    match stage {
        CoefficientExpansionStage::Admission => "Admitting endpoint powers",
        CoefficientExpansionStage::RegularSeries => "Expanding regular coefficients",
        CoefficientExpansionStage::Naming => "Naming coordinate-dependent coefficients",
        CoefficientExpansionStage::Endpoint => "Composing endpoint terms",
        CoefficientExpansionStage::Composition => "Combining Laurent coefficients",
        CoefficientExpansionStage::Coverage => "Checking Laurent coverage",
        CoefficientExpansionStage::Lowering => "Resolving coefficient requests",
        CoefficientExpansionStage::PhysicalFallback => "Using physical endpoint subtraction",
        CoefficientExpansionStage::Complete => "Coefficient expansion complete",
    }
}

#[cfg(test)]
mod tests;
