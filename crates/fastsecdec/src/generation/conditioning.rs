//! Coordinate profiles for the existing numerical precision heuristic.
//! The mapped bound dominates retained remainder rows on the open unit cube;
//! it does not certify floating-point error or exact coefficient cancellation.

use super::{
    ConditioningBasis, EndpointCancellationSource, EndpointProfileRow, GenerationError,
    GenerationOptions, mapping::MappedTerm, subtraction::endpoints,
};
use symbolica::atom::Symbol;

pub(super) struct Profile {
    pub degree: usize,
    pub rows: Vec<Vec<usize>>,
    pub basis: ConditioningBasis,
    pub endpoint_profiles: Vec<EndpointProfileRow>,
}

impl Profile {
    pub(super) fn retained(
        rows: Vec<Vec<usize>>,
        endpoint_profiles: Vec<EndpointProfileRow>,
        dimension: usize,
    ) -> Result<Self, GenerationError> {
        if rows.iter().any(|row| row.len() != dimension) {
            return Err(GenerationError::Invariant(
                "conditioning profile dimension mismatch".into(),
            ));
        }
        Ok(Self {
            degree: endpoints::checked_cancellation_degree(&rows)?,
            rows,
            basis: ConditioningBasis::RetainedRemainders,
            endpoint_profiles,
        })
    }

    pub(super) fn mapped(
        terms: &[MappedTerm],
        dimension: usize,
        regulator: Symbol,
        options: &GenerationOptions,
    ) -> Result<Self, GenerationError> {
        let mut row = vec![0; dimension];
        let mut axes = vec![Vec::new(); dimension];
        for term in terms {
            if term.powers.len() != dimension {
                return Err(GenerationError::Invariant(
                    "conditioning profile dimension mismatch".into(),
                ));
            }
            for ((maximum, sources), power) in row.iter_mut().zip(&mut axes).zip(&term.powers) {
                let endpoint =
                    endpoints::admit(power, regulator, options.max_subtractions_per_axis)?;
                *maximum = (*maximum).max(endpoint.subtractions);
                if endpoint.subtractions != 0 {
                    sources.push(EndpointCancellationSource {
                        order: endpoint.subtractions,
                        original_power: (-endpoint.constant).to_string(),
                    });
                }
            }
        }
        let rows = vec![row];
        for sources in &mut axes {
            sources.sort();
            sources.dedup();
        }
        Ok(Self {
            degree: endpoints::checked_cancellation_degree(&rows)?,
            rows,
            basis: ConditioningBasis::MappedEndpointBound,
            endpoint_profiles: vec![EndpointProfileRow { axes }],
        })
    }
}
