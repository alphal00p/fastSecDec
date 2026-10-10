//! Optional context evidence after terminal numerical failure. Unchecked
//! successful sampling never decodes or maps these saved arithmetic programs.
use crate::{
    contour::functions::dynamic::requests::Bundle,
    kernel::{KernelError, NativeProgramDescriptor, program, recipe::DynamicCheckOutput},
};
use std::{collections::BTreeSet, sync::Arc};
use symbolica::{
    atom::Symbol,
    domains::{
        float::{ComplexBall, Float, FloatLike, RealBall},
        rational::Rational,
    },
};
#[cfg(test)]
mod tests;

struct Source {
    chart: Option<usize>,
    namespace: String,
    projection: Vec<Option<usize>>,
    faces: BTreeSet<Vec<(usize, u8)>>,
    raw: Arc<[u8]>,
    output_count: usize,
    causal: usize,
    leading: usize,
}

/// A detached sector retains only its referenced raw arithmetic and compact
/// associations, rather than keeping unrelated charts or helper owners alive.
pub(in crate::kernel) struct FailureContext {
    sources: Vec<Source>,
    runtime: Vec<f64>,
    dimension: usize,
    bits: u32,
}
impl FailureContext {
    pub(in crate::kernel) fn new(
        descriptor: &NativeProgramDescriptor,
        parameters: &[Symbol],
        bundles: &BTreeSet<Bundle>,
        runtime: &[f64],
        maximum_bits: u32,
    ) -> Result<Arc<Self>, KernelError> {
        let certificates = descriptor.certificates().ok_or_else(|| {
            KernelError::Contour("dynamic failure context lacks certificates".into())
        })?;
        let mut sources = Vec::new();
        for saved in certificates {
            let faces = bundles
                .iter()
                .flat_map(|bundle| &bundle.0)
                .filter(|request| request.namespace == saved.namespace)
                .map(|request| request.face.clone())
                .collect::<BTreeSet<_>>();
            if faces.is_empty() {
                continue;
            }
            let projection = saved
                .coordinates
                .iter()
                .map(|name| {
                    Symbol::parse(name, "fastsecdec::artifact")
                        .map(|symbol| parameters.iter().position(|parameter| *parameter == symbol))
                        .map_err(KernelError::Artifact)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let slot = |kind| {
                saved
                    .schema
                    .iter()
                    .position(|value| *value == kind)
                    .ok_or_else(|| {
                        KernelError::Contour(
                            "dynamic failure context lacks a causal primitive".into(),
                        )
                    })
            };
            sources.push(Source {
                chart: saved.chart_index,
                namespace: saved.namespace.clone(),
                projection,
                faces,
                raw: Arc::from(saved.raw_program.clone()),
                output_count: saved.schema.len(),
                causal: slot(DynamicCheckOutput::CausalRay)?,
                leading: slot(DynamicCheckOutput::LeadingCausalMagnitude)?,
            });
        }
        Ok(Arc::new(Self {
            sources,
            runtime: runtime.to_vec(),
            dimension: parameters.len(),
            bits: maximum_bits.min(192),
        }))
    }

    #[cold]
    pub(in crate::kernel) fn describe(&self, point: &[f64]) -> Option<String> {
        if self.sources.is_empty() {
            return None;
        }
        Some(match self.stationary(point) {
            Ok(Some(evidence)) => evidence,
            Ok(None) => "retained causal context does not certify a stationary zero".into(),
            Err(error) => format!("causal context diagnosis unavailable: {error}"),
        })
    }

    fn stationary(&self, point: &[f64]) -> Result<Option<String>, KernelError> {
        if point.len() != self.dimension
            || point
                .iter()
                .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
        {
            return Err(KernelError::InvalidPoint);
        }
        let bits = self.bits;
        let zero = ComplexBall::from_rational_ball(
            &symbolica::domains::float::Complex::new(Rational::from(0), Rational::from(0)),
            &Rational::from(0),
            bits,
        );
        let real = |value| {
            ComplexBall::new(
                RealBall::exact(Float::with_val(bits, value)),
                RealBall::exact(Float::with_val(bits, 0)),
            )
        };
        for source in &self.sources {
            let exact = program::decode(&source.raw)?;
            crate::kernel::contour::admit_certified_arithmetic(&exact.export_instructions())?;
            if exact.get_input_len() != source.projection.len() + self.runtime.len() + 1
                || exact.get_output_len() != source.output_count
            {
                return Err(KernelError::Contour(
                    "cold causal program has a different input/output schema".into(),
                ));
            }
            let mut evaluator = exact.map_coeff_with_prec(
                &|value| ComplexBall::from_rational_ball(value, &Rational::from(0), bits),
                bits,
            );
            let mut input = vec![zero.clone(); source.projection.len() + self.runtime.len() + 1];
            for (target, coordinate) in input.iter_mut().zip(&source.projection) {
                *target = match coordinate {
                    Some(index) => real(point[*index]),
                    None => ComplexBall::new(
                        RealBall::from_rational_bounds(
                            &Rational::from(0),
                            &Rational::from(1),
                            bits,
                        ),
                        RealBall::exact(Float::with_val(bits, 0)),
                    ),
                };
            }
            for (target, value) in input[source.projection.len()..]
                .iter_mut()
                .zip(&self.runtime)
            {
                *target = real(*value);
            }
            // The final saved input is lambda. Zero asks for the original F,
            // without solving a radius or changing the production map.
            let mut output = vec![zero.clone(); source.output_count];
            for face in &source.faces {
                for (axis, value) in face {
                    input[*axis] = real(f64::from(*value));
                }
                evaluator.evaluate(&input, &mut output);
                if output[source.causal].is_fully_zero() && output[source.leading].is_fully_zero() {
                    let mut coordinates = source
                        .projection
                        .iter()
                        .map(|index| index.map(|index| point[index]))
                        .collect::<Vec<_>>();
                    for (axis, value) in face {
                        coordinates[*axis] = Some(f64::from(*value));
                    }
                    return Ok(Some(format!(
                        "retained causal context certifies F=0 and weighted-gradient magnitude=0 at {bits} bits: chart {:?}, namespace {}, full-source coordinates {coordinates:?} (None encloses [0,1]), face {face:?}; context evidence does not identify the unique failing density branch",
                        source.chart, source.namespace
                    )));
                }
                // Each face is evaluated from the actual projected point;
                // restrictions from another request must not leak into it.
                for (axis, _) in face {
                    input[*axis] = match source.projection[*axis] {
                        Some(index) => real(point[index]),
                        None => ComplexBall::new(
                            RealBall::from_rational_bounds(
                                &Rational::from(0),
                                &Rational::from(1),
                                bits,
                            ),
                            RealBall::exact(Float::with_val(bits, 0)),
                        ),
                    };
                }
            }
        }
        Ok(None)
    }
}
