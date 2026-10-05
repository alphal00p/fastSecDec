//! Validated factorized densities shared by graph and direct input.

use std::collections::BTreeSet;

use fastsecdec_sectors::{ParametricDomain, PolynomialSupport, SectorError};
use symbolica::atom::{Atom, AtomCore, Symbol};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParametricError {
    #[error("invalid parametric integral: {0}")]
    Invalid(String),
    #[error(transparent)]
    Support(#[from] SectorError),
}

/// Whether a factor must be monomialized before endpoint subtraction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FactorRole {
    /// A potentially noninteger power, such as a Symanzik U or F factor.
    Singularity,
    /// A polynomial numerator with a nonnegative integer power.
    Polynomial,
}

#[derive(Clone, Debug)]
pub struct PolynomialFactor {
    polynomial: Atom,
    exponent: Atom,
    role: FactorRole,
}

impl PolynomialFactor {
    /// Declare a factor; its polynomial and role are validated when the complete
    /// parameter list is supplied to [`ParametricIntegrand::new`].
    pub fn new(polynomial: Atom, exponent: Atom, role: FactorRole) -> Self {
        Self {
            polynomial,
            exponent,
            role,
        }
    }

    pub fn polynomial(&self) -> &Atom {
        &self.polynomial
    }
    pub fn exponent(&self) -> &Atom {
        &self.exponent
    }
    pub fn role(&self) -> FactorRole {
        self.role
    }
    pub fn support(&self, parameters: &[Symbol]) -> Result<PolynomialSupport, ParametricError> {
        polynomial_support(&self.polynomial, parameters)
    }
}

#[derive(Clone, Debug)]
pub struct ParametricTerm {
    prefactor: Atom,
    monomial_powers: Vec<Atom>,
    factors: Vec<PolynomialFactor>,
}

impl ParametricTerm {
    /// Powers belong to the original density `prod(x_i^a_i)`; they exclude
    /// Jacobians subsequently introduced by sector transformations.
    pub fn new(
        prefactor: Atom,
        monomial_powers: Vec<Atom>,
        factors: Vec<PolynomialFactor>,
    ) -> Self {
        Self {
            prefactor,
            monomial_powers,
            factors,
        }
    }

    pub fn prefactor(&self) -> &Atom {
        &self.prefactor
    }
    pub fn monomial_powers(&self) -> &[Atom] {
        &self.monomial_powers
    }
    pub fn factors(&self) -> &[PolynomialFactor] {
        &self.factors
    }
}

/// A sum of scalar densities in one declared domain and regulator.
///
/// The measure is `delta(1-sum(x)) prod(dx)` for a projective simplex and
/// `prod(dx)` for a cube or positive orthant. Every projective term is verified
/// to have degree minus the number of parameters before any gauge fixing.
#[derive(Clone, Debug)]
pub struct ParametricIntegrand {
    parameters: Vec<Symbol>,
    regulator: Symbol,
    domain: ParametricDomain,
    terms: Vec<ParametricTerm>,
}

impl ParametricIntegrand {
    pub fn new(
        parameters: Vec<Symbol>,
        regulator: Symbol,
        domain: ParametricDomain,
        terms: Vec<ParametricTerm>,
    ) -> Result<Self, ParametricError> {
        let invalid = |message: &str| ParametricError::Invalid(message.into());
        if parameters.iter().collect::<BTreeSet<_>>().len() != parameters.len() {
            return Err(invalid("parameters must be distinct symbols"));
        }
        if parameters.contains(&regulator) {
            return Err(invalid(
                "the regulator cannot also be an integration parameter",
            ));
        }
        if domain == ParametricDomain::ProjectiveSimplex && parameters.is_empty() {
            return Err(invalid(
                "a projective simplex requires at least one parameter",
            ));
        }
        let atoms = parameters.iter().map(|s| Atom::var(*s)).collect::<Vec<_>>();
        let dependent = |expression: &Atom| atoms.iter().any(|p| expression.contains(p.as_view()));
        let regulator_atom = Atom::var(regulator);
        let mut retained = Vec::with_capacity(terms.len());
        for term in terms {
            if term.monomial_powers.len() != parameters.len() {
                return Err(invalid(
                    "one monomial density power is required per parameter",
                ));
            }
            if dependent(&term.prefactor) || term.monomial_powers.iter().any(&dependent) {
                return Err(invalid(
                    "prefactors and monomial exponents must be parameter independent",
                ));
            }
            let mut degree = term.monomial_powers.iter().cloned().sum::<Atom>();
            let mut zero = term.prefactor.is_zero();
            for factor in &term.factors {
                if dependent(&factor.exponent) {
                    return Err(invalid("factor exponents must be parameter independent"));
                }
                if factor.role == FactorRole::Polynomial {
                    let power = u64::try_from(factor.exponent.as_view()).map_err(|_| {
                        invalid("polynomial numerator factors require nonnegative integer powers")
                    })?;
                    zero |= power != 0 && factor.polynomial.is_zero();
                } else {
                    if factor.polynomial.contains(regulator_atom.as_view()) {
                        return Err(invalid(
                            "singularity polynomials must be regulator independent",
                        ));
                    }
                    if factor.polynomial.is_zero() {
                        return Err(invalid(
                            "an identically zero singularity polynomial needs scalelessness analysis",
                        ));
                    }
                }
                if factor.polynomial.is_zero() || factor.exponent.is_zero() {
                    continue;
                }
                if domain == ParametricDomain::ProjectiveSimplex
                    && factor.role == FactorRole::Polynomial
                    && let Some(factor_degree) =
                        super::homogeneity::degree(&factor.polynomial, &parameters)
                {
                    degree += Atom::num(factor_degree) * &factor.exponent;
                    continue;
                }
                // Native factored recognition avoids materializing a numerator's
                // support merely to admit it on a nonprojective domain. Compound
                // indeterminates are allowed only as parameter-independent
                // coefficients; otherwise sparse conversion supplies the exact
                // fallback. Singular projective factors still require support
                // below; regular factors first use native scaling admission.
                if domain != ParametricDomain::ProjectiveSimplex
                    && factor
                        .polynomial
                        .is_polynomial(true, false)
                        .is_some_and(|indeterminates| {
                            indeterminates.iter().all(|indeterminate| {
                                atoms.iter().any(|p| *indeterminate == p.as_view())
                                    || atoms.iter().all(|p| !indeterminate.contains(p.as_view()))
                            })
                        })
                {
                    continue;
                }
                let support = polynomial_support(&factor.polynomial, &parameters)?;
                if domain == ParametricDomain::ProjectiveSimplex {
                    let degrees = support
                        .exponents()
                        .iter()
                        .map(|powers| {
                            powers.iter().fold(
                                numerica::domains::integer::Integer::from(0),
                                |sum, power| sum + power,
                            )
                        })
                        .collect::<BTreeSet<numerica::domains::integer::Integer>>();
                    if degrees.len() != 1 {
                        return Err(invalid("each projective factor must be homogeneous"));
                    }
                    degree += Atom::num(degrees.into_iter().next().unwrap()) * &factor.exponent;
                }
            }
            if zero {
                continue;
            }
            if domain == ParametricDomain::ProjectiveSimplex
                && !(degree + Atom::num(parameters.len())).expand().is_zero()
            {
                return Err(invalid(
                    "each projective density must have total degree minus its parameter count",
                ));
            }
            retained.push(term);
        }
        Ok(Self {
            parameters,
            regulator,
            domain,
            terms: retained,
        })
    }

    pub fn parameters(&self) -> &[Symbol] {
        &self.parameters
    }
    pub fn regulator(&self) -> Symbol {
        self.regulator
    }
    pub fn domain(&self) -> ParametricDomain {
        self.domain
    }
    pub fn terms(&self) -> &[ParametricTerm] {
        &self.terms
    }

    pub fn density(&self) -> Atom {
        self.terms
            .iter()
            .map(|term| {
                let monomial = self
                    .parameters
                    .iter()
                    .zip(&term.monomial_powers)
                    .map(|(parameter, power)| Atom::var(*parameter).pow(power))
                    .product::<Atom>();
                &term.prefactor
                    * monomial
                    * term
                        .factors
                        .iter()
                        .map(|factor| factor.polynomial.pow(&factor.exponent))
                        .product::<Atom>()
            })
            .sum()
    }
}

/// Bridge existing Symbolica polynomial data to the exact geometry crate.
/// Expressions hidden in coefficients are checked, preventing rational or
/// functional parameter dependence from being mistaken for a polynomial.
pub fn polynomial_support(
    expression: &Atom,
    parameters: &[Symbol],
) -> Result<PolynomialSupport, ParametricError> {
    let variables = parameters.iter().map(|s| Atom::var(*s)).collect::<Vec<_>>();
    let polynomial = expression.to_polynomial_in_vars::<u32>(&variables);
    let mut exponents = Vec::with_capacity(polynomial.nterms());
    for term in &polynomial {
        if variables
            .iter()
            .any(|p| term.coefficient.contains(p.as_view()))
        {
            return Err(ParametricError::Invalid(
                "factor is not polynomial in the integration parameters".into(),
            ));
        }
        exponents.push(
            term.exponents
                .iter()
                .map(|power| i64::from(*power))
                .collect(),
        );
    }
    Ok(PolynomialSupport::new(exponents)?)
}
