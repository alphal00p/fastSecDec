//! Select the coefficient representation before common multiplicity/assembly.
//! This module owns route observations and exclusive phase timings only.

use super::{
    CoefficientExpansionMethod, CoefficientExpansionStage, CoefficientRequestCounts,
    GenerationError, GenerationEvent, GenerationOptions, GenerationPhase, GenerationProgress,
    coefficient_first::{self, Progress, RequestStage, Route, ScalarKind},
    conditioning::Profile,
    context::emit,
    laurent::{self, TemplateCache},
    mapping::MappedTerm,
    subtraction,
};
use std::{collections::BTreeMap, ops::ControlFlow, time::Instant};
use symbolica::atom::{AliasedAtom, Symbol};

pub(super) struct Output {
    pub coefficients: BTreeMap<i32, AliasedAtom>,
    pub conditioning: Profile,
    pub phase: GenerationPhase,
    pub phase_started: Instant,
}

pub(super) struct Representative<'a> {
    pub parameters: &'a [Symbol],
    pub regulator: Symbol,
    pub index: usize,
    pub total: usize,
}

pub(super) fn expand(
    mapped: Vec<MappedTerm>,
    representative: Representative<'_>,
    options: &GenerationOptions,
    templates: &mut TemplateCache,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<Output, GenerationError> {
    match options.coefficient_expansion.method {
        CoefficientExpansionMethod::Physical => {
            physical(mapped, representative, options, templates, progress)
        }
        CoefficientExpansionMethod::NativeNamed => {
            named(mapped, representative, options, templates, progress)
        }
    }
}

fn physical(
    mapped: Vec<MappedTerm>,
    representative: Representative<'_>,
    options: &GenerationOptions,
    templates: &mut TemplateCache,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<Output, GenerationError> {
    let Representative {
        parameters,
        regulator,
        index,
        total,
    } = representative;
    let started = Instant::now();
    let subtracted = subtraction::subtract_profiled(mapped, parameters, regulator, options)?;
    let expression = subtracted.expression;
    let terms = subtracted.count;
    let conditioning = Profile::retained(
        subtracted.cancellation_terms,
        subtracted.endpoint_profiles,
        parameters.len(),
    )?;
    emit(
        progress,
        GenerationProgress::PhaseTiming {
            phase: GenerationPhase::Subtraction,
            seconds: started.elapsed().as_secs_f64(),
        },
    )?;
    emit(
        progress,
        GenerationProgress::Subtraction {
            sector: index,
            total,
            terms,
        },
    )?;
    emit(
        progress,
        GenerationProgress::LaurentExpansion {
            sector: index,
            total,
        },
    )?;
    let started = Instant::now();
    let coefficients = laurent::expand(
        &expression,
        parameters,
        regulator,
        options.max_order,
        templates,
    )?;
    Ok(Output {
        coefficients,
        conditioning,
        phase: GenerationPhase::Laurent,
        phase_started: started,
    })
}

#[derive(Default)]
struct Observation {
    attempt: usize,
    width: i64,
    formal_pieces: usize,
    requests: CoefficientRequestCounts,
}

impl Observation {
    fn requests(&mut self, counts: coefficient_first::RequestCounts) {
        self.requests = CoefficientRequestCounts {
            source_bodies: counts.source_bodies,
            unique_requests: counts.unique_requests,
            cached_partials: counts.cached_partials,
            aliases: counts.aliases,
            interleaved_requests: counts.interleaved_requests,
            fallback_requests: counts.fallback_requests,
        };
    }

    fn event(
        &self,
        representative: &Representative<'_>,
        stage: CoefficientExpansionStage,
    ) -> GenerationProgress {
        GenerationProgress::CoefficientExpansion {
            sector: representative.index,
            total: representative.total,
            stage,
            attempt: self.attempt,
            relative_width: self.width,
            formal_pieces: self.formal_pieces,
            requests: self.requests,
        }
    }

    fn observe(&mut self, progress: Progress) -> CoefficientExpansionStage {
        use CoefficientExpansionStage as Stage;
        match progress {
            Progress::Admission { .. } => Stage::Admission,
            Progress::Attempt { number, width } => {
                *self = Self {
                    attempt: number,
                    width,
                    ..Self::default()
                };
                Stage::RegularSeries
            }
            Progress::RegularSeries { .. } => Stage::RegularSeries,
            Progress::ScalarSeries {
                kind: ScalarKind::EndpointInverse,
            }
            | Progress::Endpoint { .. }
            | Progress::EndpointDegree { .. } => Stage::Endpoint,
            Progress::ScalarSeries { .. }
            | Progress::ComposePiece { .. }
            | Progress::ComposeGroup { .. }
            | Progress::EmptySeries => Stage::Composition,
            Progress::Coverage {
                attempt,
                width,
                absolute_order,
                formal_pieces,
            } => {
                // Retain the native cutoff internally; public status does not
                // approximate or serialize it as a float/string.
                let _ = absolute_order;
                self.attempt = attempt;
                self.width = width;
                self.formal_pieces = formal_pieces;
                Stage::Coverage
            }
            Progress::Requests(request) => {
                self.requests(request.counts);
                match request.stage {
                    RequestStage::ReserveSymbols => Stage::Admission,
                    RequestStage::NameCoefficients => Stage::Naming,
                    RequestStage::LowerRequests => Stage::Lowering,
                }
            }
            Progress::PhysicalFallback => Stage::PhysicalFallback,
        }
    }
}

fn named(
    mapped: Vec<MappedTerm>,
    representative: Representative<'_>,
    options: &GenerationOptions,
    templates: &mut TemplateCache,
    progress: &mut impl FnMut(&GenerationEvent) -> ControlFlow<()>,
) -> Result<Output, GenerationError> {
    let started = Instant::now();
    let mut observation = Observation::default();
    let limits = &options.coefficient_expansion;
    let completed = coefficient_first::expand(
        &mapped,
        representative.parameters,
        representative.regulator,
        options,
        coefficient_first::Limits {
            max_attempts: limits.max_series_attempts,
            max_relative_width: limits.max_relative_width,
            requests: coefficient_first::RequestLimits {
                max_unique_requests: limits.max_unique_requests,
            },
        },
        templates,
        &mut |work| {
            let stage = observation.observe(work);
            progress(&observation.event(&representative, stage).into())
        },
    )?;
    observation.requests(completed.requests);
    let conditioning = match completed.route {
        Route::Native {
            attempts,
            relative_width,
            absolute_order,
            formal_pieces,
        } => {
            if absolute_order <= options.max_order {
                return Err(GenerationError::Invariant(
                    "named coefficient result lacks native coverage".into(),
                ));
            }
            observation.attempt = attempts;
            observation.width = relative_width;
            observation.formal_pieces = formal_pieces;
            Profile::mapped(
                &mapped,
                representative.parameters.len(),
                representative.regulator,
                options,
            )?
        }
        Route::PhysicalFallback {
            pieces,
            cancellation_terms,
        } => {
            debug_assert!(cancellation_terms.len() <= pieces);
            // This is an actual physical piece count, not a named formal count.
            observation.formal_pieces = 0;
            Profile::retained(
                cancellation_terms,
                completed.endpoint_profiles,
                representative.parameters.len(),
            )?
        }
    };
    emit(
        progress,
        observation.event(&representative, CoefficientExpansionStage::Complete),
    )?;
    Ok(Output {
        coefficients: completed.coefficients,
        conditioning,
        phase: GenerationPhase::CoefficientExpansion,
        phase_started: started,
    })
}
