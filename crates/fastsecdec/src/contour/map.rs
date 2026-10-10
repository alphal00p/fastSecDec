//! Native Symbolica construction of an endpoint-preserving causal chart.
use super::{ContourDefinitions, ContourMetadata, functions::causal_log, lambda_symbol};
use crate::{generation::GenerationError, parametric::FactorSemantics};
use std::sync::Arc;
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    id::{Pattern, Replacement},
};

#[derive(Clone, Debug)]
pub struct FixedContourMap {
    map: SmoothContourMap,
}

impl FixedContourMap {
    pub fn new(parameters: &[Symbol], causal_polynomial: Atom) -> Result<Self, GenerationError> {
        Ok(Self {
            map: SmoothContourMap::new(parameters, causal_polynomial, Atom::var(lambda_symbol()))?,
        })
    }

    pub fn metadata(&self) -> &ContourMetadata {
        self.map.metadata()
    }

    pub fn substitute(&self, expression: &Atom) -> Atom {
        self.map.substitute(expression)
    }

    pub(crate) fn into_inner(self) -> SmoothContourMap {
        self.map
    }
}

/// Shared native geometry for a caller-proved real positive smooth strength.
/// Dynamic recipes retain their independent descriptor and coefficient
/// definitions alongside this geometry. The shared map does not determine the
/// saved mathematical recipe or its runtime validation policy.
#[derive(Clone, Debug)]
pub(crate) struct SmoothContourMap {
    parameters: Vec<Symbol>,
    metadata: ContourMetadata,
}
impl SmoothContourMap {
    pub(crate) fn new(
        parameters: &[Symbol],
        causal_polynomial: Atom,
        strength: Atom,
    ) -> Result<Self, GenerationError> {
        Self::with_definitions(parameters, causal_polynomial, strength, Arc::default())
    }

    pub(crate) fn with_definitions(
        parameters: &[Symbol],
        causal_polynomial: Atom,
        strength: Atom,
        definitions: Arc<ContourDefinitions>,
    ) -> Result<Self, GenerationError> {
        definitions
            .validate(false)
            .map_err(GenerationError::Contour)?;
        definitions
            .select([&strength])
            .map_err(GenerationError::Contour)?;
        if parameters
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != parameters.len()
            || parameters.contains(&lambda_symbol())
            || causal_polynomial.contains(Atom::var(lambda_symbol()).as_view())
        {
            return Err(GenerationError::Contour(
                "contour coordinates must be distinct and disjoint from the reserved strength"
                    .into(),
            ));
        }
        crate::parametric::polynomial_support(&causal_polynomial, parameters)?;
        if !crate::kernel::is_real_expression(
            &causal_polynomial,
            &causal_polynomial
                .get_all_symbols(true)
                .into_iter()
                .collect::<Vec<_>>(),
        ) {
            return Err(GenerationError::Contour(
                "the contour F polynomial must be real for real inputs".into(),
            ));
        }
        let imaginary = Atom::i();
        let ratios = parameters
            .iter()
            .map(|parameter| {
                Atom::one()
                    - &imaginary
                        * &strength
                        * (Atom::one() - Atom::var(*parameter))
                        * causal_polynomial.derivative(*parameter)
            })
            .collect::<Vec<_>>();
        let images = parameters
            .iter()
            .zip(&ratios)
            .map(|(parameter, ratio)| Atom::var(*parameter) * ratio)
            .collect::<Vec<_>>();
        let dimension = u32::try_from(parameters.len())
            .map_err(|_| GenerationError::ResourceLimit("contour Jacobian dimension"))?;
        let entries = images
            .iter()
            .flat_map(|image| parameters.iter().map(|p| image.derivative(*p)))
            .collect();
        let jacobian = super::determinant::determinant(entries, dimension)?;
        Ok(Self {
            parameters: parameters.to_vec(),
            metadata: ContourMetadata {
                definitions,
                causal_polynomial,
                positive_polynomials: Vec::new(),
                images,
                ratios,
                jacobian,
                validation_faces: vec![vec![]],
            },
        })
    }

    pub(crate) fn metadata(&self) -> &ContourMetadata {
        &self.metadata
    }

    pub(crate) fn substitute(&self, expression: &Atom) -> Atom {
        expression.replace_multiple(self.parameters.iter().zip(&self.metadata.images).map(
            |(source, target)| {
                Replacement::new(
                    Pattern::Literal(Atom::var(*source)),
                    Pattern::Literal(target.clone()),
                )
            },
        ))
    }

    /// Density after removal of the real endpoint monomials. Each branch is
    /// represented separately before Laurent expansion and endpoint derivatives.
    pub(crate) fn smooth_density(
        &mut self,
        powers: &[Atom],
        factors: &[(Atom, Atom, FactorSemantics)],
    ) -> Atom {
        let mut density = self.metadata.jacobian.clone();
        for (ratio, power) in self.metadata.ratios.iter().zip(powers) {
            density *= continued_power(ratio, power, false);
        }
        for (residual, exponent, semantics) in factors {
            if *semantics == FactorSemantics::Positive
                && !self.metadata.positive_polynomials.contains(residual)
            {
                self.metadata.positive_polynomials.push(residual.clone());
            }
            density *= continued_power(
                &self.substitute(residual),
                exponent,
                *semantics == FactorSemantics::Causal,
            );
        }
        density
    }
}

pub(crate) fn continued_power(base: &Atom, exponent: &Atom, causal: bool) -> Atom {
    if symbolica::domains::integer::Integer::try_from(exponent.as_view()).is_ok() {
        base.pow(exponent)
    } else {
        let logarithm = if causal { causal_log(base) } else { base.log() };
        (exponent * logarithm).exp()
    }
}
