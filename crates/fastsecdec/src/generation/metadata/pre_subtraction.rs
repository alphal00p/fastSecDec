//! Compact native facts retained at the mapped-density boundary.
use super::super::{GenerationError, mapping::MappedTerm, subtraction::endpoints};
use symbolica::{
    atom::{Atom, Symbol},
    domains::rational::Rational,
};

#[derive(Clone, Debug)]
pub struct EndpointPower {
    pub(crate) exponent: Atom,
    pub(crate) constant: Rational,
    pub(crate) slope: Rational,
    pub(crate) subtraction_count: usize,
}
impl EndpointPower {
    pub fn exponent(&self) -> &Atom {
        &self.exponent
    }
    /// The exact `b` in `x^(b + c epsilon)`.
    pub fn constant(&self) -> &Rational {
        &self.constant
    }
    pub fn slope(&self) -> &Rational {
        &self.slope
    }
    /// Taylor coefficients required by endpoint power admission. Vanishing
    /// boundary coefficients can still remove a term; this is not a pole count.
    pub fn subtraction_count(&self) -> usize {
        self.subtraction_count
    }
    pub(crate) fn admit(
        exponent: Atom,
        regulator: Symbol,
        maximum: usize,
    ) -> Result<Self, GenerationError> {
        let admission = endpoints::admit(&exponent, regulator, maximum)?;
        Ok(Self {
            exponent,
            constant: admission.constant,
            slope: admission.slope,
            subtraction_count: admission.subtractions,
        })
    }
}

#[derive(Clone, Debug)]
pub struct PreSubtractionTerm {
    pub(crate) prefactor: Atom,
    pub(crate) powers: Vec<EndpointPower>,
    pub(crate) regular_expression_bytes: usize,
}
impl PreSubtractionTerm {
    /// Coordinate-independent native factor, including regulator dependence.
    pub fn prefactor(&self) -> &Atom {
        &self.prefactor
    }
    pub fn powers(&self) -> &[EndpointPower] {
        &self.powers
    }
    /// Native Atom storage size at mapping, not expanded monomial count or
    /// evaluator size. The potentially large regular body is not duplicated.
    pub fn regular_expression_bytes(&self) -> usize {
        self.regular_expression_bytes
    }
}

#[derive(Clone, Debug)]
pub struct PreSubtractionMetadata {
    pub(crate) regulator: Symbol,
    pub(crate) terms: Vec<PreSubtractionTerm>,
}
impl PreSubtractionMetadata {
    pub fn version(&self) -> u32 {
        1
    }
    pub fn regulator(&self) -> Symbol {
        self.regulator
    }
    pub fn terms(&self) -> &[PreSubtractionTerm] {
        &self.terms
    }
    pub(in crate::generation) fn capture(
        terms: &[MappedTerm],
        regulator: Symbol,
        maximum: usize,
    ) -> Result<Self, GenerationError> {
        Ok(Self {
            regulator,
            terms: terms
                .iter()
                .map(|term| {
                    Ok(PreSubtractionTerm {
                        prefactor: term.prefactor.clone(),
                        powers: term
                            .powers
                            .iter()
                            .cloned()
                            .map(|power| EndpointPower::admit(power, regulator, maximum))
                            .collect::<Result<_, _>>()?,
                        regular_expression_bytes: term.regular.as_view().get_byte_size(),
                    })
                })
                .collect::<Result<_, GenerationError>>()?,
        })
    }
}
