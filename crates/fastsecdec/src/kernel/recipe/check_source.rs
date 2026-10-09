//! Generation-only source for independent certified checks. Final dynamic
//! admission will consume optimized saved programs, never rebuild this source.
use crate::contour::dynamic::DynamicEnvelope;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    id::{Pattern, Replacement},
};

#[derive(Clone, Debug, PartialEq, Eq, bincode::Encode, bincode::Decode)]
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
    pub(crate) parameters: Vec<Symbol>,
    pub(crate) schema: Vec<DynamicCheckOutput>,
    pub(crate) outputs: Vec<Atom>,
}

impl DynamicCheckSource {
    pub(super) fn validate_for(
        &self,
        chart: &super::DynamicChartRecipe,
    ) -> Result<(), super::KernelError> {
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
        if self.chart_index != chart.chart_index
            || self.parameters.len() != chart.dimension
            || self.schema != expected
            || self.outputs.len() != expected.len()
        {
            return Err(super::invalid(
                "dynamic check source schema differs from its full-sector descriptor",
            ));
        }
        Ok(())
    }

    /// This is the same full-sector direction at every face. The independent
    /// lambda input permits ball enclosures without executing production roots.
    pub(crate) fn from_envelope(chart_index: usize, envelope: &DynamicEnvelope) -> Self {
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
        let mut result = Self {
            chart_index,
            parameters: envelope.parameters().to_vec(),
            schema: Vec::new(),
            outputs: Vec::new(),
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
        result
    }
}
