//! Saved native polynomial programs for optional causal checks. Runtime never
//! reconstructs a symbolic map or repeats evaluator optimization. Certified
//! signs concern the evaluated points only; a finite fixed-strength pilot is
//! deliberately not presented as a proof for the whole integration domain.
//! The native ball owner currently certifies arithmetic and integer powers,
//! so unsupported constants/functions remain an explicit optional-check
//! limitation. They do not prohibit unchecked evaluation of the integrand.
//! Enclosures certify the symbolic polynomial map at supplied floating inputs,
//! not every rounded intermediate in a separately compiled production kernel.
use super::super::{CompilationSettings, KernelError, KernelSet, program};
use crate::{
    contour::{ContourSettings, ContourValidation, ContourValidationOptions},
    generation::GenerationMetadata,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::float::Complex,
    domains::{
        float::{ComplexBall, Float, FloatLike, RealBall},
        rational::Rational,
    },
    evaluate::{ExportedInstructions, ExpressionEvaluator, Instruction},
    id::{Pattern, Replacement},
};

#[derive(Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub(in crate::kernel) struct CheckProgram {
    pub chart_index: usize,
    pub program: Vec<u8>,
    /// Optional checks must not prohibit a valid unchecked contour integrand.
    pub unsupported: Option<String>,
}

pub(in crate::kernel) fn build_checks(
    metadata: &GenerationMetadata,
    runtime: &[Symbol],
    settings: CompilationSettings,
) -> Result<Vec<CheckProgram>, KernelError> {
    metadata
        .charts()
        .iter()
        .filter(|chart| chart.source_index() == chart.representative())
        .filter_map(|chart| chart.contour().map(|contour| (chart, contour)))
        .map(|(chart, contour)| {
            let coordinates = chart.coordinates().target_parameters();
            let replacements = coordinates
                .iter()
                .zip(contour.images())
                .map(|(x, z)| (Atom::var(*x), z.clone()))
                .collect::<Vec<_>>();
            let rules = replacements
                .iter()
                .map(|(x, z)| {
                    Replacement::new(Pattern::Literal(x.clone()), Pattern::Literal(z.clone()))
                })
                .collect::<Vec<_>>();
            let deform = |expression: &Atom| expression.replace_multiple(&rules);
            let mut outputs = vec![deform(contour.causal_polynomial())];
            outputs.extend(contour.images().iter().cloned());
            outputs.extend(contour.positive_polynomials().iter().map(deform));
            let inputs = coordinates
                .iter()
                .chain(runtime)
                .map(|x| Atom::var(*x))
                .collect::<Vec<_>>();
            let functions = contour
                .function_definitions()
                .function_map(&outputs)
                .map_err(KernelError::Compilation)?;
            let exact = Atom::evaluator_multiple(&outputs, &inputs)
                .function_map(functions)
                .optimization_settings(settings.native())
                .build()
                .map_err(|e| KernelError::Compilation(e.to_string()))?;
            let unsupported = admit_certified_arithmetic(&exact.export_instructions())
                .err()
                .map(|error| error.to_string());
            Ok(CheckProgram {
                chart_index: chart.source_index(),
                program: program::encode(&exact)?,
                unsupported,
            })
        })
        .collect()
}

/// This checks eligibility for the owner's *certifying arithmetic subset*;
/// it is not a replacement native-IR structural validator.
pub(in crate::kernel) fn admit_certified_arithmetic(
    program: &ExportedInstructions<Complex<Rational>>,
) -> Result<(), KernelError> {
    if !program.constant_functions.is_empty() {
        return Err(KernelError::Compilation(
            "contour checks require polynomial arithmetic, without external constants".into(),
        ));
    }
    for instruction in &program.instructions {
        match instruction {
            Instruction::IfElse(..)
            | Instruction::Goto(..)
            | Instruction::Label(..)
            | Instruction::Join(..) => {
                return Err(KernelError::Compilation(
                    "conditional control flow is not certified contour-check arithmetic".into(),
                ));
            }
            Instruction::Powf(..) => {
                return Err(KernelError::Compilation(
                    "noninteger powers are not certified contour-check arithmetic".into(),
                ));
            }
            Instruction::Fun(_, function, _) => {
                if !program
                    .sub_evaluators
                    .iter()
                    .any(|f| f.symbol == function.0 && f.tags == function.1)
                {
                    return Err(KernelError::Compilation(
                        "external/transcendental functions are not certified contour-check arithmetic"
                            .into(),
                    ));
                }
            }
            Instruction::Add(..)
            | Instruction::Mul(..)
            | Instruction::Pow(..)
            | Instruction::Assign(..) => {}
        }
    }
    for body in &program.sub_evaluators {
        admit_certified_arithmetic(&body.instructions)?;
    }
    Ok(())
}

#[cfg(test)]
mod certified_domain_tests {
    use super::*;
    use symbolica::{evaluate::Slot, symbol};

    #[test]
    fn certified_contour_programs_reject_conditional_control_flow() {
        let x = Atom::var(symbol!("contour_certified_domain_test::x"));
        let program = x
            .evaluator(std::slice::from_ref(&x))
            .build()
            .unwrap()
            .export_instructions();
        assert!(admit_certified_arithmetic(&program).is_ok());
        // Test domain admission directly; these instructions are never
        // evaluated. A valid conditional evaluator needs branch enclosure
        // semantics that the owner's scalar ball domain does not promise.
        for instruction in [
            Instruction::IfElse(Slot::Param(0), 0),
            Instruction::Goto(0),
            Instruction::Label(0),
            Instruction::Join(Slot::Out(0), Slot::Param(0), Slot::Param(0), Slot::Param(0)),
            Instruction::Powf(Slot::Out(0), Slot::Param(0), Slot::Param(0), true),
            Instruction::Fun(
                Slot::Out(0),
                Box::new((
                    symbol!("contour_certified_domain_test::external"),
                    vec![],
                    vec![Slot::Param(0)],
                )),
                true,
            ),
        ] {
            let mut altered = program.clone();
            altered.instructions.push(instruction);
            assert!(admit_certified_arithmetic(&altered).is_err());
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContourValidationChart {
    pub chart_index: usize,
    pub kernel_sector: Option<usize>,
    /// Complete stochastic associations. Older records use `kernel_sector`.
    #[serde(default)]
    pub kernel_sectors: Vec<usize>,
    /// The same source may contribute both stochastic and exact terms.
    #[serde(default)]
    pub includes_exact: bool,
    pub dimension: usize,
}
impl ContourValidationChart {
    /// One native scope decision for complete source associations, including
    /// legacy snapshots whose only association is the singular field.
    pub fn required_by_scope(&self, scope: &crate::results::ResultScope) -> bool {
        use crate::results::{ExactContributionPolicy, ResultScope};
        match scope {
            ResultScope::FullIntegral => true,
            ResultScope::SelectedSectors {
                sector_ids,
                exact_policy,
            } => {
                let exact = self.includes_exact
                    || (self.kernel_sectors.is_empty() && self.kernel_sector.is_none());
                let sectors = self
                    .kernel_sectors
                    .iter()
                    .copied()
                    .chain(self.kernel_sector);
                (exact && *exact_policy == ExactContributionPolicy::IncludeAll)
                    || sectors
                        .into_iter()
                        .any(|sector| sector_ids.contains(&(sector as u64)))
            }
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ContourCheckReport {
    pub chart_index: usize,
    pub checked_arguments: usize,
    pub maximum_bits: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContourValidationReport {
    pub policy: ContourValidation,
    /// Scope represented by this snapshot; completion applies to this list.
    pub required_charts: Vec<usize>,
    pub validated_charts: Vec<usize>,
    pub pilot_complete: bool,
    pub accepted_pilot_points: usize,
    pub checked_arguments: usize,
    pub maximum_bits: u32,
    pub production_checked_arguments: usize,
    pub production_maximum_bits: u32,
}

/// Counters observed by one resident evaluator owner. Independent workers
/// return these snapshots for aggregation; a coordinator's unused clone does
/// not stand in for their work.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContourProductionReport {
    pub policy: ContourValidation,
    pub checked_arguments: usize,
    pub maximum_bits: u32,
}

#[derive(Clone)]
pub(in crate::kernel) struct Check {
    chart_index: usize,
    kernel_sector: Option<usize>,
    dimension: usize,
    positive_count: usize,
    faces: Vec<Vec<(usize, u8)>>,
    unsupported: Option<String>,
    exact: program::ExactProgram,
    numeric: Option<(u32, ExpressionEvaluator<ComplexBall>)>,
    runtime: Vec<f64>,
    lambda_index: usize,
    pilot_points: usize,
    checked_arguments: usize,
    maximum_bits: u32,
}

impl Check {
    fn evaluate(
        &mut self,
        point: &[f64],
        homotopy: bool,
    ) -> Result<ContourCheckReport, KernelError> {
        if let Some(reason) = &self.unsupported {
            return Err(KernelError::Contour(format!(
                "certified validation is unavailable for chart {}: {reason}; use runtime validation off only if independently justified",
                self.chart_index
            )));
        }
        if point.len() != self.dimension {
            return Err(KernelError::Dimension {
                expected: self.dimension,
                actual: point.len(),
            });
        }
        if point
            .iter()
            .any(|x| !x.is_finite() || !(0.0..=1.0).contains(x))
        {
            return Err(KernelError::InvalidPoint);
        }
        let mut result = ContourCheckReport {
            chart_index: self.chart_index,
            ..Default::default()
        };
        for face in self.faces.clone() {
            let mut argument = point.to_vec();
            for (axis, value) in face {
                argument[axis] = f64::from(value);
            }
            if self.positive_count > 0 {
                let bits = self.evaluate_argument(&argument, 0.0)?;
                result.maximum_bits = result.maximum_bits.max(bits);
                result.checked_arguments += 1;
                self.maximum_bits = self.maximum_bits.max(bits);
                self.checked_arguments += 1;
            }
            let fractions: &[f64] = if homotopy {
                &[0.125, 0.25, 0.5, 1.0]
            } else {
                &[1.0]
            };
            for fraction in fractions {
                let bits = self.evaluate_argument(&argument, *fraction)?;
                result.maximum_bits = result.maximum_bits.max(bits);
                result.checked_arguments += 1;
                self.maximum_bits = self.maximum_bits.max(bits);
                self.checked_arguments += 1;
            }
        }
        Ok(result)
    }

    fn evaluate_argument(&mut self, point: &[f64], fraction: f64) -> Result<u32, KernelError> {
        for bits in [96, 192, 384, 768, 1536] {
            if self.numeric.as_ref().is_none_or(|entry| entry.0 != bits) {
                let numeric = self.exact.clone().map_coeff_with_prec(
                    &|c| ComplexBall::from_rational_ball(c, &Rational::zero(), bits),
                    bits,
                );
                self.numeric = Some((bits, numeric));
            }
            let real = point
                .iter()
                .chain(&self.runtime)
                .copied()
                .collect::<Vec<_>>();
            let mut input = real
                .iter()
                .map(|x| {
                    ComplexBall::new(
                        RealBall::exact(Float::with_val(bits, *x)),
                        RealBall::exact(Float::with_val(bits, 0)),
                    )
                })
                .collect::<Vec<_>>();
            input[self.dimension + self.lambda_index].re *=
                RealBall::exact(Float::with_val(bits, fraction));
            let zero = ComplexBall::new(
                RealBall::exact(Float::with_val(bits, 0)),
                RealBall::exact(Float::with_val(bits, 0)),
            );
            let mut output = vec![zero; 1 + self.dimension + self.positive_count];
            self.numeric
                .as_mut()
                .unwrap()
                .1
                .evaluate(&input, &mut output);
            let causal = &output[0];
            if fraction != 0.0 && causal.im.is_strictly_positive() {
                return Err(self.failure(
                    point,
                    fraction,
                    "F has a certified positive imaginary part",
                ));
            }
            let stationary = output[1..1 + self.dimension]
                .iter()
                .all(|z| z.im.is_fully_zero());
            let causal_ok = fraction == 0.0
                || causal.im.is_strictly_negative()
                || (stationary && causal.im.is_fully_zero() && !causal.contains_zero());
            let mut positive_ok = true;
            for (index, value) in output[1 + self.dimension..].iter().enumerate() {
                if value.re.is_strictly_negative() {
                    return Err(self.failure(
                        point,
                        fraction,
                        &format!("positive factor U[{index}] has a certified negative real part"),
                    ));
                }
                positive_ok &= value.re.is_strictly_positive();
            }
            if causal_ok && positive_ok {
                return Ok(bits);
            }
        }
        Err(self.failure(point, fraction, "causal/positive-factor condition unresolved at 1536 bits (including possible stationary F=0)"))
    }
    fn failure(&self, point: &[f64], fraction: f64, reason: &str) -> KernelError {
        KernelError::Contour(format!(
            "chart {}, sector {:?}, x={point:?}, runtime={:?}, homotopy fraction={fraction}: {reason}",
            self.chart_index, self.kernel_sector, self.runtime
        ))
    }
}

pub(in crate::kernel) struct FixedBinding {
    settings: ContourSettings,
    checks: Vec<Check>,
    ready: BTreeMap<usize, Arc<AtomicBool>>,
    point: BTreeMap<Symbol, f64>,
}
impl Clone for FixedBinding {
    fn clone(&self) -> Self {
        Self {
            settings: self.settings.clone(),
            checks: self.checks.clone(),
            ready: self
                .ready
                .iter()
                .map(|(chart, ready)| {
                    (
                        *chart,
                        Arc::new(AtomicBool::new(ready.load(Ordering::Acquire))),
                    )
                })
                .collect(),
            point: self.point.clone(),
        }
    }
}

impl FixedBinding {
    pub fn require_ready(&self, scope: &crate::results::ResultScope) -> Result<(), KernelError> {
        use crate::results::{ExactContributionPolicy, ResultScope};
        if self.settings.validation.policy == ContourValidation::Off {
            return Ok(());
        }
        for check in &self.checks {
            let required = match scope {
                ResultScope::FullIntegral => true,
                ResultScope::SelectedSectors {
                    sector_ids,
                    exact_policy,
                } => check.kernel_sector.map_or(
                    *exact_policy == ExactContributionPolicy::IncludeAll,
                    |sector| sector_ids.contains(&(sector as u64)),
                ),
            };
            if required && !self.ready[&check.chart_index].load(Ordering::Acquire) {
                return Err(KernelError::Contour(format!(
                    "complete the configured contour pilot for chart {} before integration",
                    check.chart_index
                )));
            }
        }
        Ok(())
    }

    pub fn settings(&self) -> &ContourSettings {
        &self.settings
    }
    pub fn new(
        kernels: &KernelSet,
        point: &BTreeMap<Symbol, f64>,
        settings: ContourSettings,
    ) -> Result<Self, KernelError> {
        let mut checks = Vec::new();
        // Indexed archives retain a shared exact record even when the entire
        // contribution is the literal zero vector. No contour map is evaluated
        // in that record, so its absent chart metadata is not missing evidence.
        // Numerical zeros or nonzero exact records do not justify this exception.
        let identically_zero = kernels.sectors.is_empty()
            && kernels
                .exact_expressions
                .iter()
                .all(|value| value.is_zero());
        if settings.validation.policy != ContourValidation::Off && !identically_zero {
            let metadata = kernels
                .metadata
                .as_ref()
                .ok_or_else(|| KernelError::Artifact("missing contour chart metadata".into()))?;
            let lambda_index = kernels
                .runtime_parameters
                .iter()
                .position(|s| *s == crate::contour::lambda_symbol())
                .ok_or_else(|| KernelError::Artifact("missing contour runtime strength".into()))?;
            let runtime = kernels
                .runtime_parameters
                .iter()
                .map(|s| {
                    point
                        .get(s)
                        .copied()
                        .ok_or_else(|| KernelError::Parameters(format!("missing value for {s}")))
                })
                .collect::<Result<Vec<_>, _>>()?;
            for saved in &kernels.contour_checks {
                let chart = metadata.charts().get(saved.chart_index).ok_or_else(|| {
                    KernelError::Artifact("validation program has no source chart".into())
                })?;
                let contour = chart.contour().ok_or_else(|| {
                    KernelError::Artifact(
                        "validation program attached to an undeformed chart".into(),
                    )
                })?;
                let dimension = chart.coordinates().target_parameters().len();
                let exact = program::decode(&saved.program)?;
                let unsupported = admit_certified_arithmetic(&exact.export_instructions())
                    .err()
                    .map(|error| error.to_string());
                if exact.get_input_len() != dimension + runtime.len()
                    || exact.get_output_len()
                        != 1 + dimension + contour.positive_polynomials().len()
                {
                    return Err(KernelError::Artifact(
                        "contour validation program layout differs".into(),
                    ));
                }
                checks.push(Check {
                    chart_index: saved.chart_index,
                    kernel_sector: chart.kernel_sector(),
                    dimension,
                    positive_count: contour.positive_polynomials().len(),
                    faces: contour.validation_faces().to_vec(),
                    unsupported,
                    exact,
                    numeric: None,
                    runtime: runtime.clone(),
                    lambda_index,
                    pilot_points: 0,
                    checked_arguments: 0,
                    maximum_bits: 0,
                });
            }
            let expected = metadata
                .charts()
                .iter()
                .filter(|c| c.source_index() == c.representative() && c.contour().is_some())
                .count();
            if checks.len() != expected {
                return Err(KernelError::Artifact(
                    "contour artifact lacks its saved validation programs; regenerate".into(),
                ));
            }
        }
        let ready = checks
            .iter()
            .map(|check| (check.chart_index, Arc::new(AtomicBool::new(false))))
            .collect();
        Ok(Self {
            settings,
            checks,
            ready,
            point: point.clone(),
        })
    }
    pub fn for_sector(&self, index: usize) -> Option<SectorValidation> {
        let readiness = self
            .checks
            .iter()
            .filter(|check| check.kernel_sector == Some(index))
            .map(|check| self.ready[&check.chart_index].clone())
            .collect();
        match self.settings.validation.policy {
            ContourValidation::Off => None,
            ContourValidation::Pilot => Some(SectorValidation::Pilot(readiness)),
            ContourValidation::Always => Some(SectorValidation::Always {
                ready: readiness,
                checks: self
                    .checks
                    .iter()
                    .filter(|c| c.kernel_sector == Some(index))
                    .map(|check| {
                        let mut check = check.clone();
                        check.pilot_points = 0;
                        check.checked_arguments = 0;
                        check.maximum_bits = 0;
                        check
                    })
                    .collect(),
            }),
        }
    }
    pub fn charts(&self) -> Vec<ContourValidationChart> {
        self.checks
            .iter()
            .map(|c| ContourValidationChart {
                chart_index: c.chart_index,
                kernel_sector: c.kernel_sector,
                kernel_sectors: c.kernel_sector.into_iter().collect(),
                includes_exact: c.kernel_sector.is_none(),
                dimension: c.dimension,
            })
            .collect()
    }
    pub fn validate_point(
        &mut self,
        chart: usize,
        point: &[f64],
        homotopy: bool,
    ) -> Result<ContourCheckReport, KernelError> {
        let check = self
            .checks
            .iter_mut()
            .find(|c| c.chart_index == chart)
            .ok_or_else(|| KernelError::Contour(format!("unknown validation chart {chart}")))?;
        let result = check.evaluate(point, homotopy)?;
        if homotopy {
            check.pilot_points += 1;
        }
        Ok(result)
    }
    pub fn finish(&mut self) -> Result<ContourValidationReport, KernelError> {
        self.finish_for_charts(
            &self
                .checks
                .iter()
                .map(|check| check.chart_index)
                .collect::<Vec<_>>(),
        )
    }
    pub fn finish_for_charts(
        &mut self,
        charts: &[usize],
    ) -> Result<ContourValidationReport, KernelError> {
        if self.settings.validation.policy == ContourValidation::Off {
            return Err(KernelError::Contour(
                "validation is disabled; no new pilot evidence was collected".into(),
            ));
        }
        for chart in charts {
            let check = self
                .checks
                .iter()
                .find(|check| check.chart_index == *chart)
                .ok_or_else(|| KernelError::Contour(format!("unknown validation chart {chart}")))?;
            if check.pilot_points < self.settings.validation.pilot_points {
                return Err(KernelError::Contour(format!(
                    "pilot lacks the requested number of validated points on chart {chart}"
                )));
            }
        }
        for chart in charts {
            self.ready[chart].store(true, Ordering::Release);
        }
        Ok(self.report_for_charts(charts))
    }
    pub fn report(&self) -> ContourValidationReport {
        self.report_for_charts(
            &self
                .checks
                .iter()
                .map(|check| check.chart_index)
                .collect::<Vec<_>>(),
        )
    }
    fn report_for_charts(&self, charts: &[usize]) -> ContourValidationReport {
        let scope = charts.iter().copied().collect::<BTreeSet<_>>();
        let checks = self
            .checks
            .iter()
            .filter(|check| scope.contains(&check.chart_index))
            .collect::<Vec<_>>();
        let required_charts = checks
            .iter()
            .map(|check| check.chart_index)
            .collect::<Vec<_>>();
        let validated_charts = required_charts
            .iter()
            .copied()
            .filter(|chart| self.ready[chart].load(Ordering::Acquire))
            .collect::<Vec<_>>();
        ContourValidationReport {
            policy: self.settings.validation.policy,
            pilot_complete: (self.settings.validation.policy != ContourValidation::Off
                || !required_charts.is_empty())
                && required_charts.len() == validated_charts.len(),
            required_charts,
            validated_charts,
            accepted_pilot_points: checks.iter().map(|c| c.pilot_points).sum(),
            checked_arguments: checks.iter().map(|c| c.checked_arguments).sum(),
            maximum_bits: checks.iter().map(|c| c.maximum_bits).max().unwrap_or(0),
            production_checked_arguments: 0,
            production_maximum_bits: 0,
        }
    }
    pub fn with_policy(
        &self,
        kernels: &KernelSet,
        validation: ContourValidationOptions,
    ) -> Result<Self, KernelError> {
        let mut settings = self.settings.clone();
        settings.validation = validation;
        settings.validate().map_err(KernelError::Parameters)?;
        let mut result =
            if self.checks.is_empty() && settings.validation.policy != ContourValidation::Off {
                Self::new(kernels, &self.point, settings.clone())?
            } else {
                self.clone()
            };
        result.settings = settings;
        for check in &result.checks {
            if check.pilot_points < result.settings.validation.pilot_points {
                result.ready[&check.chart_index].store(false, Ordering::Release);
            }
        }
        Ok(result)
    }
}

#[derive(Clone)]
pub(in crate::kernel) enum SectorValidation {
    Always {
        ready: Vec<Arc<AtomicBool>>,
        checks: Vec<Check>,
    },
    Pilot(Vec<Arc<AtomicBool>>),
    /// Dynamic numeric owners perform their own attempt-local checks.
    Dynamic {
        ready: Vec<Arc<AtomicBool>>,
        policy: ContourValidation,
    },
}
impl SectorValidation {
    pub fn take_report(&mut self) -> ContourProductionReport {
        let report = self.report();
        if let Self::Always { checks, .. } = self {
            for check in checks {
                check.checked_arguments = 0;
                check.maximum_bits = 0;
            }
        }
        report
    }
    pub fn fork(&self) -> Self {
        let mut result = self.clone();
        if let Self::Always { checks, .. } = &mut result {
            for check in checks {
                check.pilot_points = 0;
                check.checked_arguments = 0;
                check.maximum_bits = 0;
            }
        }
        result
    }
    pub fn report(&self) -> ContourProductionReport {
        let (checked_arguments, maximum_bits) = self.statistics();
        ContourProductionReport {
            policy: match self {
                Self::Always { .. } => ContourValidation::Always,
                Self::Pilot(_) => ContourValidation::Pilot,
                Self::Dynamic { policy, .. } => *policy,
            },
            checked_arguments,
            maximum_bits,
        }
    }
    pub fn statistics(&self) -> (usize, u32) {
        match self {
            Self::Always { checks, .. } => (
                checks.iter().map(|c| c.checked_arguments).sum(),
                checks.iter().map(|c| c.maximum_bits).max().unwrap_or(0),
            ),
            Self::Pilot(_) | Self::Dynamic { .. } => (0, 0),
        }
    }
    pub fn validate(&mut self, point: &[f64]) -> Result<(), KernelError> {
        let ready = match self {
            Self::Always { ready, .. } | Self::Pilot(ready) | Self::Dynamic { ready, .. } => ready,
        };
        if !ready.iter().all(|ready| ready.load(Ordering::Acquire)) {
            return Err(KernelError::Contour(
                "complete the configured contour pilot before production evaluation".into(),
            ));
        }
        match self {
            Self::Always { checks, .. } => {
                for check in checks {
                    check.evaluate(point, false)?;
                }
                Ok(())
            }
            Self::Pilot(_) | Self::Dynamic { .. } => Ok(()),
        }
    }
}
