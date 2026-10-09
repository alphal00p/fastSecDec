//! Source identities use Symbolica's canonical representation, not its
//! process-local serialization tables. They do not perform decomposition.
use super::GenerationError;
use crate::{
    kernel::RuntimeMassConstraint,
    parametric::{FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand},
};
use serde::{Serialize, Serializer};
use symbolica::atom::{Atom, AtomCore, Symbol};

struct CanonicalAtom<'a>(&'a Atom);
impl Serialize for CanonicalAtom<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_canonical_string())
    }
}

struct CanonicalSymbol(Symbol);
impl Serialize for CanonicalSymbol {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&Atom::var(self.0).to_canonical_string())
    }
}

#[derive(Serialize)]
struct Factor<'a> {
    polynomial: CanonicalAtom<'a>,
    exponent: CanonicalAtom<'a>,
    role: &'static str,
    semantics: FactorSemantics,
}

#[derive(Serialize)]
struct Term<'a> {
    prefactor: CanonicalAtom<'a>,
    powers: Vec<CanonicalAtom<'a>>,
    factors: Vec<Factor<'a>>,
}

#[derive(Serialize)]
struct Source<'a> {
    domain: &'static str,
    parameters: Vec<CanonicalSymbol>,
    regulator: CanonicalSymbol,
    runtime_parameters: Vec<CanonicalSymbol>,
    runtime_mass_constraints: Vec<(&'a str, CanonicalAtom<'a>)>,
    terms: Vec<Term<'a>>,
}

/// Identify an ordered physical input without generating or compiling it.
///
/// All expressions and symbol attributes use Symbolica's native canonical
/// printing. Unlike native transport bytes, this is independent of unrelated
/// symbol registration and polynomial coefficient-ring tables. The identity
/// retains term/factor order and distinguishes representations; it is not a
/// proof that differently supplied integrals are mathematically equivalent.
///
/// Runtime inputs retain their declared order, excluding the reserved contour
/// mathematical inputs. Their numerical values belong to later bound-integrand
/// identities. Scheduling, generation settings, target coordinates, contour
/// recipes, strengths and caps are not part of this source identity.
pub fn source_identity(
    input: &ParametricIntegrand,
    runtime_parameters: &[Symbol],
    runtime_mass_constraints: &[RuntimeMassConstraint],
) -> Result<String, GenerationError> {
    let reserved = [
        crate::contour::lambda_symbol(),
        crate::contour::dynamic::safety_fraction_symbol(),
        crate::contour::dynamic::lambda_cap_symbol(),
        crate::contour::dynamic::displacement_cap_symbol(),
        crate::contour::dynamic::radius_fraction_symbol(),
    ];
    let source = Source {
        domain: match input.domain() {
            ParametricDomain::ProjectiveSimplex => "projective_simplex",
            ParametricDomain::UnitCube => "unit_cube",
            ParametricDomain::PositiveOrthant => "positive_orthant",
        },
        parameters: input
            .parameters()
            .iter()
            .copied()
            .map(CanonicalSymbol)
            .collect(),
        regulator: CanonicalSymbol(input.regulator()),
        runtime_parameters: runtime_parameters
            .iter()
            .copied()
            .filter(|symbol| !reserved.contains(symbol))
            .map(CanonicalSymbol)
            .collect(),
        runtime_mass_constraints: runtime_mass_constraints
            .iter()
            .map(|constraint| {
                (
                    constraint.name.as_str(),
                    CanonicalAtom(&constraint.expression),
                )
            })
            .collect(),
        terms: input
            .terms()
            .iter()
            .map(|term| Term {
                prefactor: CanonicalAtom(term.prefactor()),
                powers: term.monomial_powers().iter().map(CanonicalAtom).collect(),
                factors: term
                    .factors()
                    .iter()
                    .map(|factor| Factor {
                        polynomial: CanonicalAtom(factor.polynomial()),
                        exponent: CanonicalAtom(factor.exponent()),
                        role: match factor.role() {
                            FactorRole::Singularity => "singularity",
                            FactorRole::Polynomial => "polynomial",
                        },
                        semantics: factor.semantics(),
                    })
                    .collect(),
            })
            .collect(),
    };
    let mut digest = blake3::Hasher::new();
    digest.update(b"fastsecdec-physical-source-v1\0");
    // Serialize one native canonical expression at a time instead of retaining
    // a second string copy of every large factor or a complete JSON document.
    serde_json::to_writer(&mut digest, &source)
        .map_err(|error| GenerationError::Invariant(format!("source identity: {error}")))?;
    Ok(digest.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests;
