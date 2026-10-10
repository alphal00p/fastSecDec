//! Generation-only source for independent certified checks. Final dynamic
//! admission will consume optimized saved programs, never rebuild this source.
use crate::contour::dynamic::DynamicEnvelope;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    id::{Pattern, Replacement},
};
mod coefficients;
pub(super) mod identity;
pub(crate) use coefficients::{CoefficientInput, CoefficientSource};
pub(crate) use identity::FactorIdentity;

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    bincode::Encode,
    bincode::Decode,
)]
pub(crate) enum DynamicCheckOutput {
    DirectionNormSquared,
    LeadingCausalMagnitude,
    CausalRay,
    CausalSquaredBound { order: u32 },
    SpectralMean { order: u32 },
    SpectralGapSquared { order: u32 },
    PositiveOriginal { factor: usize },
    PositiveRay { factor: usize },
    PositiveRayCoefficient { factor: usize, order: u32 },
    PositiveHarmfulCoefficient { factor: usize, order: u32 },
}

#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
#[bincode(decode_context = "symbolica::state::StateMap")]
pub(crate) struct DynamicCheckSource {
    pub(crate) chart_index: usize,
    #[bincode(with_serde)]
    pub(crate) recipe: crate::kernel::ProgramRecipe,
    pub(crate) namespace: String,
    pub(crate) factors: FactorIdentity,
    pub(crate) full_strength: Atom,
    pub(crate) parameters: Vec<Symbol>,
    pub(crate) schema: Vec<DynamicCheckOutput>,
    pub(crate) outputs: Vec<Atom>,
    pub(crate) coefficients: CoefficientSource,
}

pub(crate) fn output_schema(chart: &super::DynamicChartRecipe) -> Vec<DynamicCheckOutput> {
    use DynamicCheckOutput::*;
    let mut expected = vec![DirectionNormSquared, LeadingCausalMagnitude, CausalRay];
    for order in &chart.causal_orders {
        expected.extend([
            CausalSquaredBound { order: *order },
            SpectralMean { order: *order },
            SpectralGapSquared { order: *order },
        ]);
    }
    for (factor, orders) in chart.positive_orders.iter().enumerate() {
        expected.extend([PositiveOriginal { factor }, PositiveRay { factor }]);
        for order in orders {
            expected.extend([
                PositiveRayCoefficient {
                    factor,
                    order: *order,
                },
                PositiveHarmfulCoefficient {
                    factor,
                    order: *order,
                },
            ]);
        }
    }
    expected
}

pub(crate) fn coefficient_schema(
    chart: &super::DynamicChartRecipe,
    recipe: crate::kernel::ProgramRecipe,
) -> Result<Vec<CoefficientInput>, super::KernelError> {
    coefficients::expected_schema(chart, recipe).map_err(super::invalid)
}

impl DynamicCheckSource {
    pub(super) fn validate_for(
        &self,
        chart: &super::DynamicChartRecipe,
    ) -> Result<(), super::KernelError> {
        let expected = output_schema(chart);
        if self.chart_index != chart.chart_index
            || self.parameters.len() != chart.dimension
            || self.schema != expected
            || self.outputs.len() != expected.len()
            || !self.recipe.is_dynamic()
            || !identity::valid_digest(&self.namespace)
        {
            return Err(super::invalid(
                "dynamic check source schema differs from its full-sector descriptor",
            ));
        }
        self.factors
            .validate(chart.positive_orders.len())
            .map_err(super::invalid)?;
        self.coefficients
            .validate_for(chart, self.recipe)
            .map_err(super::invalid)?;
        if identity::namespace_for_chart(self.recipe, &self.parameters, &self.factors, chart)
            .map_err(super::invalid)?
            != self.namespace
        {
            return Err(super::invalid(
                "dynamic request namespace differs from its mathematical recipe",
            ));
        }
        let strength = self.full_strength.as_fun_view().ok_or_else(|| {
            super::invalid("dynamic check source lacks its full-strength callback")
        })?;
        if strength.get_symbol().get_name() != "fastsecdec::contour::dynamic::strength_v1"
            || strength.get_nargs() != chart.coefficient_count + 4
            || strength.get(0) != Atom::num(chart.coefficient_count).as_view()
        {
            return Err(super::invalid(
                "dynamic check source full-strength schema differs from descriptor",
            ));
        }
        Ok(())
    }

    /// This is the same full-sector direction at every face. The independent
    /// lambda input permits ball enclosures without executing production roots.
    pub(crate) fn from_envelope(
        chart_index: usize,
        envelope: &DynamicEnvelope,
        recipe: crate::kernel::ProgramRecipe,
        full_strength: Atom,
    ) -> Result<Self, super::KernelError> {
        let lambda = Atom::var(crate::contour::lambda_symbol());
        let rules = envelope
            .parameters()
            .iter()
            .zip(envelope.direction())
            .map(|(x, v)| {
                Replacement::new(
                    Pattern::Literal(Atom::var(*x)),
                    Pattern::Literal(Atom::var(*x) - Atom::i() * &lambda * v),
                )
            })
            .collect::<Vec<_>>();
        let factors = FactorIdentity::from_envelope(envelope).map_err(super::invalid)?;
        let namespace = identity::namespace(recipe, envelope, &factors).map_err(super::invalid)?;
        let mut result = Self {
            chart_index,
            recipe,
            namespace,
            factors,
            full_strength,
            parameters: envelope.parameters().to_vec(),
            schema: Vec::new(),
            outputs: Vec::new(),
            coefficients: CoefficientSource::new(envelope, recipe).map_err(super::invalid)?,
        };
        let mut push = |kind, expression| {
            result.schema.push(kind);
            result.outputs.push(expression);
        };
        push(
            DynamicCheckOutput::DirectionNormSquared,
            envelope.direction().iter().map(|v| v.pow(2)).sum(),
        );
        push(
            DynamicCheckOutput::LeadingCausalMagnitude,
            envelope.leading_causal_magnitude().clone(),
        );
        push(
            DynamicCheckOutput::CausalRay,
            envelope.causal_polynomial().replace_multiple(&rules),
        );
        for term in envelope.causal_terms() {
            let order = term.order();
            push(
                DynamicCheckOutput::CausalSquaredBound { order },
                term.squared_bound().clone(),
            );
            push(
                DynamicCheckOutput::SpectralMean { order },
                term.spectral_mean().clone(),
            );
            push(
                DynamicCheckOutput::SpectralGapSquared { order },
                term.spectral_gap_squared().clone(),
            );
        }
        for (factor, positive) in envelope.positive_factors().iter().enumerate() {
            push(
                DynamicCheckOutput::PositiveOriginal { factor },
                positive.polynomial().clone(),
            );
            push(
                DynamicCheckOutput::PositiveRay { factor },
                positive.polynomial().replace_multiple(&rules),
            );
            for term in positive.terms() {
                let order = term.order();
                push(
                    DynamicCheckOutput::PositiveRayCoefficient { factor, order },
                    term.ray_coefficient().clone(),
                );
                push(
                    DynamicCheckOutput::PositiveHarmfulCoefficient { factor, order },
                    term.positive_bound().argument().clone(),
                );
            }
        }
        Ok(result)
    }
}
