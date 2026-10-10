//! First certified rational interval bridge, not a general resolver.
mod atlas;
mod bound;
pub use bound::BoundContinuation;
mod normalize;

use super::{
    gcad::{DomainOrigin, GcadError, VerifiedDecomposition},
    maps::{CellMap, ParameterAdmission},
    phase::CausalCell,
};
use crate::{
    generation::{GenerationOptions, endpoint_power_with_regulators},
    parametric::FactorRole,
};
use std::{collections::BTreeMap, ops::ControlFlow, sync::Arc};
use symbolica::{
    atom::{Atom, AtomCore, Symbol, SymbolAttribute, SymbolBuilder},
    domains::{atom::AtomField, rational::Rational},
    evaluate::{FunctionMap, FunctionRegistrationOptions, InliningPolicy},
    id::{Pattern, Replacement},
    wrap_symbol,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid rational endpoint bridge: {0}")]
    Invalid(String),
    #[error("unsupported rational endpoint bridge: {0}")]
    Unsupported(String),
    #[error("rational endpoint bridge cancelled")]
    Cancelled,
    #[error("rational endpoint bridge resource limit: {0}")]
    ResourceIncomplete(&'static str),
    #[error(transparent)]
    Geometry(#[from] GcadError),
    #[error(transparent)]
    Generation(#[from] crate::generation::GenerationError),
}
pub type Result<T> = std::result::Result<T, Error>;

/// Strict bounds for a common open real-epsilon interval. The intersection is
/// established before finite chart seams are omitted or continuation starts.
#[derive(Clone, Debug, Default)]
pub struct ConvergenceStrip {
    lower: Option<Rational>,
    upper: Option<Rational>,
}
impl ConvergenceStrip {
    pub fn lower(&self) -> Option<&Rational> {
        self.lower.as_ref()
    }
    pub fn upper(&self) -> Option<&Rational> {
        self.upper.as_ref()
    }
    fn admit(&mut self, power: &Atom, eps: Symbol) -> Result<()> {
        let (constant, slopes) = endpoint_power_with_regulators(power, &[eps])
            .map_err(|e| unsupported(&e.to_string()))?;
        let slope = &slopes[0];
        if slope.is_zero() {
            if constant <= -1 {
                return Err(unsupported(
                    "no common epsilon convergence strip for an unregulated endpoint",
                ));
            }
            return Ok(());
        }
        let boundary = (Rational::from(-1) - constant) / slope;
        if slope > &Rational::zero() {
            if self.lower.as_ref().is_none_or(|l| l < &boundary) {
                self.lower = Some(boundary);
            }
        } else if self.upper.as_ref().is_none_or(|u| u > &boundary) {
            self.upper = Some(boundary);
        }
        if let (Some(l), Some(u)) = (&self.lower, &self.upper)
            && l >= u
        {
            return Err(unsupported(
                "cell family has no common epsilon convergence strip",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_charts: usize,
    pub max_degree: u32,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_charts: 128,
            max_degree: 256,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Progress {
    Cell(usize),
    Chart(usize),
    Factor {
        chart: usize,
        term: usize,
        factor: usize,
    },
    Continue(usize),
}
fn poll(progress: Progress, observer: &mut impl FnMut(Progress) -> ControlFlow<()>) -> Result<()> {
    if observer(progress).is_break() {
        Err(Error::Cancelled)
    } else {
        Ok(())
    }
}
fn substitute(expression: &Atom, values: &BTreeMap<Symbol, Rational>) -> Atom {
    expression.replace_multiple(values.iter().map(|(s, v)| {
        Replacement::new(
            Pattern::Literal(Atom::var(*s)),
            Pattern::Literal(Atom::num(v.clone())),
        )
    }))
}
fn exact(expression: &Atom) -> Result<Rational> {
    Rational::try_from(expression.cancel().as_view())
        .map_err(|e| unsupported(&format!("expected exact rational fiber expression: {e}")))
}
fn invalid(message: &str) -> Error {
    Error::Invalid(message.into())
}
fn unsupported(message: &str) -> Error {
    Error::Unsupported(message.into())
}

#[derive(Clone, Debug)]
pub struct ClosedUnit {
    pub(super) original_factor: Option<usize>,
    pub(super) multiplicity: u32,
    pub(super) residual: Atom,
    pub(super) lower_value: Rational,
    pub(super) upper_value: Rational,
}
impl ClosedUnit {
    pub fn factor_index(&self) -> Option<usize> {
        self.original_factor
    }
    pub fn multiplicity(&self) -> u32 {
        self.multiplicity
    }
    pub fn residual(&self) -> &Atom {
        &self.residual
    }
    pub fn endpoint_values(&self) -> (&Rational, &Rational) {
        (&self.lower_value, &self.upper_value)
    }
}
#[derive(Clone, Debug)]
struct NormalizedTerm {
    prefactor: Atom,
    regular: Atom,
    power: Atom,
    units: Vec<ClosedUnit>,
}
#[derive(Debug)]
pub struct IntervalChart {
    admission: ParameterAdmission,
    image: Atom,
    measure: Atom,
    orientation: i8,
    bounds: (Rational, Rational),
    terms: Vec<NormalizedTerm>,
}
impl IntervalChart {
    pub fn admission(&self) -> &ParameterAdmission {
        &self.admission
    }
    pub fn image(&self) -> &Atom {
        &self.image
    }
    pub fn positive_measure(&self) -> &Atom {
        &self.measure
    }
    pub fn orientation(&self) -> i8 {
        self.orientation
    }
    pub fn interval(&self) -> (&Rational, &Rational) {
        (&self.bounds.0, &self.bounds.1)
    }
    pub fn units(&self, term: usize) -> Option<&[ClosedUnit]> {
        self.terms.get(term).map(|t| t.units.as_slice())
    }
}
/// The constructor owns the coverage, unit and strip proofs. No raw tuple or
/// supplied boolean can create this capability.
#[derive(Debug)]
pub struct RegularizedFiber {
    owner: Arc<VerifiedDecomposition>,
    parameters: BTreeMap<Symbol, Rational>,
    unit: Symbol,
    numerator: Symbol,
    numerator_bodies: Vec<Atom>,
    charts: Vec<IntervalChart>,
    strip: ConvergenceStrip,
}
impl RegularizedFiber {
    pub fn admit(
        owner: Arc<VerifiedDecomposition>,
        parameters: BTreeMap<Symbol, Rational>,
        unit: Symbol,
        limits: Limits,
        mut observer: impl FnMut(Progress) -> ControlFlow<()>,
    ) -> Result<Self> {
        atlas::admit(owner, parameters, unit, limits, &mut observer)
    }
    pub fn decomposition(&self) -> &Arc<VerifiedDecomposition> {
        &self.owner
    }
    pub fn parameters(&self) -> &BTreeMap<Symbol, Rational> {
        &self.parameters
    }
    pub fn charts(&self) -> &[IntervalChart] {
        &self.charts
    }
    pub fn convergence_strip(&self) -> &ConvergenceStrip {
        &self.strip
    }
    pub fn continue_symbolically(
        &self,
        options: &GenerationOptions,
        mut observer: impl FnMut(Progress) -> ControlFlow<()>,
    ) -> Result<ContinuedFiber<'_>> {
        if options.mode != crate::generation::GenerationMode::Symbolic
            || options.contour_enabled()
            || options.contour_jacobian != crate::contour::ContourJacobian::Symbolic
            || options.source_sectors.is_some()
        {
            return Err(unsupported(
                "continuation requires symbolic endpoints without a contour or source-sector selection",
            ));
        }
        let mut expressions = Vec::with_capacity(self.charts.len());
        let mut profiles = Vec::with_capacity(self.charts.len());
        for (index, chart) in self.charts.iter().enumerate() {
            poll(Progress::Continue(index), &mut observer)?;
            let continued = crate::generation::threshold_subtract(
                chart
                    .terms
                    .iter()
                    .map(|term| {
                        (
                            term.prefactor.clone(),
                            term.regular.clone(),
                            vec![term.power.clone()],
                        )
                    })
                    .collect(),
                &[self.unit],
                &[self.owner.request().input().regulator()],
                options,
            )
            .map_err(Error::Generation)?;
            profiles.push(continued.endpoint_profiles);
            expressions.push(continued.expression);
        }
        // Never discard individual cell poles. This native sum is the object
        // passed to a later Laurent pass and coherent evaluator.
        let expression = expressions.iter().cloned().sum();
        Ok(ContinuedFiber {
            certificate: self,
            expression,
            chart_expressions: expressions,
            profiles,
            functions: self.functions(options.max_subtractions_per_axis, false, &mut observer)?,
            derivative_order: options.max_subtractions_per_axis,
        })
    }
    fn functions(
        &self,
        degree: usize,
        bind_parameters: bool,
        observer: &mut impl FnMut(Progress) -> ControlFlow<()>,
    ) -> Result<FunctionMap> {
        let source = self.owner.request().domain().coordinates()[0];
        let mut functions = FunctionMap::new();
        for (term, body) in self.numerator_bodies.iter().enumerate() {
            let body = if bind_parameters {
                let value = substitute(body, &self.parameters);
                bound::no_physical_parameters(&value, self)?;
                value
            } else {
                body.clone()
            };
            functions
                .add_tagged_function(
                    self.numerator,
                    vec![Atom::num(term)],
                    vec![source],
                    body.clone(),
                )
                .map_err(|e| invalid(&e.to_string()))?;
            let mut partial = body.clone();
            for order in 1..=degree {
                poll(Progress::Continue(term), observer)?;
                partial = partial.derivative(source);
                functions
                    .add_tagged_function_with_options(
                        Symbol::DERIVATIVE,
                        vec![
                            Atom::Zero,
                            Atom::num(order),
                            Atom::var(self.numerator),
                            Atom::num(term),
                        ],
                        vec![source],
                        partial.clone(),
                        FunctionRegistrationOptions::new().inlining(InliningPolicy::Always),
                    )
                    .map_err(|e| invalid(&e.to_string()))?;
            }
        }
        Ok(functions)
    }
}
pub struct ContinuedFiber<'a> {
    certificate: &'a RegularizedFiber,
    expression: Atom,
    chart_expressions: Vec<Atom>,
    profiles: Vec<Vec<crate::generation::EndpointProfileRow>>,
    functions: FunctionMap,
    derivative_order: usize,
}
impl ContinuedFiber<'_> {
    pub fn certificate(&self) -> &RegularizedFiber {
        self.certificate
    }
    pub fn expression(&self) -> &Atom {
        &self.expression
    }
    pub fn chart_expressions(&self) -> &[Atom] {
        &self.chart_expressions
    }
    pub fn profiles(&self) -> &[Vec<crate::generation::EndpointProfileRow>] {
        &self.profiles
    }
    /// Low-level native compiler input, not an independently rebound program.
    /// Keep this certificate alive and supply exactly its admitted parameters.
    /// No unchecked caller value, raw function map, or bare expression carries
    /// the coverage/unit/strip proof. A later owned compiler must enforce this.
    pub fn functions(&self) -> &FunctionMap {
        &self.functions
    }
}

#[cfg(test)]
mod tests;
