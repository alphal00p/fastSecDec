//! Shared native input DTO used by ordinary and threshold staging records.
use super::codec::{Atoms, StreamingError, invalid};
use crate::parametric::{
    FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
    PolynomialFactor,
};
use serde::{Deserialize, Serialize};
use symbolica::atom::Symbol;

// Preserve the existing ordinary-source metadata layout and Atom insertion order.
#[derive(Serialize, Deserialize)]
struct Factor {
    polynomial: usize,
    exponent: usize,
    polynomial_role: bool,
    #[serde(default)]
    semantics: FactorSemantics,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct Term {
    prefactor: usize,
    powers: Vec<usize>,
    factors: Vec<Factor>,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct Input {
    pub parameters: usize,
    pub domain: u8,
    pub terms: Vec<Term>,
}
impl Input {
    pub fn encode(input: &ParametricIntegrand, atoms: &mut Atoms) -> Self {
        Self {
            parameters: input.parameters().len(),
            domain: match input.domain() {
                ParametricDomain::ProjectiveSimplex => 0,
                ParametricDomain::UnitCube => 1,
                ParametricDomain::PositiveOrthant => 2,
            },
            terms: input
                .terms()
                .iter()
                .map(|t| Term {
                    prefactor: atoms.push(t.prefactor()),
                    powers: t.monomial_powers().iter().map(|v| atoms.push(v)).collect(),
                    factors: t
                        .factors()
                        .iter()
                        .map(|f| Factor {
                            polynomial: atoms.push(f.polynomial()),
                            exponent: atoms.push(f.exponent()),
                            polynomial_role: f.role() == FactorRole::Polynomial,
                            semantics: f.semantics(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
    /// Symbols are ordered input coordinates, followed by the regulator.
    pub fn decode(
        self,
        atoms: &Atoms,
        symbols: &[Symbol],
    ) -> Result<ParametricIntegrand, StreamingError> {
        if self.parameters.checked_add(1) != Some(symbols.len()) {
            return Err(invalid("source symbol layout mismatch"));
        }
        let domain = match self.domain {
            0 => ParametricDomain::ProjectiveSimplex,
            1 => ParametricDomain::UnitCube,
            2 => ParametricDomain::PositiveOrthant,
            _ => return Err(invalid("source domain")),
        };
        let terms = self
            .terms
            .into_iter()
            .map(|t| {
                Ok(ParametricTerm::new(
                    atoms.take(t.prefactor)?,
                    t.powers
                        .into_iter()
                        .map(|i| atoms.take(i))
                        .collect::<Result<_, _>>()?,
                    t.factors
                        .into_iter()
                        .map(|f| {
                            Ok(PolynomialFactor::new(
                                atoms.take(f.polynomial)?,
                                atoms.take(f.exponent)?,
                                if f.polynomial_role {
                                    FactorRole::Polynomial
                                } else {
                                    FactorRole::Singularity
                                },
                            )
                            .with_semantics(f.semantics))
                        })
                        .collect::<Result<_, StreamingError>>()?,
                ))
            })
            .collect::<Result<_, StreamingError>>()?;
        ParametricIntegrand::new(
            symbols[..self.parameters].to_vec(),
            symbols[self.parameters],
            domain,
            terms,
        )
        .map_err(crate::generation::GenerationError::from)
        .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::{
        GenerationOptions,
        streaming::{codec, records},
    };
    use symbolica::{atom::Atom, parse, symbol};

    // Frozen pre-extraction source DTO. This test is a transport compatibility
    // oracle, not an alternate production serializer.
    #[derive(Serialize)]
    struct LegacyFactor {
        polynomial: usize,
        exponent: usize,
        polynomial_role: bool,
        semantics: FactorSemantics,
    }
    #[derive(Serialize)]
    struct LegacyTerm {
        prefactor: usize,
        powers: Vec<usize>,
        factors: Vec<LegacyFactor>,
    }
    #[derive(Serialize)]
    struct LegacySource {
        #[serde(skip_serializing_if = "Option::is_none")]
        source_scope: Option<crate::generation::GenerationSourceScope>,
        source_identity: String,
        parameters: usize,
        targets: usize,
        runtime_parameters: usize,
        runtime_mass_constraints: Vec<(String, usize)>,
        domain: u8,
        terms: Vec<LegacyTerm>,
        options: records::Options,
    }
    #[test]
    fn extracted_input_preserves_ordinary_source_metadata_and_native_record_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let input = ParametricIntegrand::new(
            vec![symbol!("staging_layout::x")],
            symbol!("staging_layout::eps"),
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(
                parse!("1+I"),
                vec![parse!("-1+staging_layout::eps")],
                vec![
                    PolynomialFactor::new(
                        parse!("1-staging_layout::a*staging_layout::x"),
                        parse!("-1-staging_layout::eps"),
                        FactorRole::Singularity,
                    )
                    .with_semantics(FactorSemantics::Causal),
                    PolynomialFactor::new(
                        parse!("1+staging_layout::x"),
                        Atom::num(2),
                        FactorRole::Polynomial,
                    ),
                ],
            )],
        )
        .unwrap();
        let options = GenerationOptions::default();
        let runtime = [symbol!("staging_layout::a")];
        let targets = [symbol!("staging_layout::t")];
        let constraints = [crate::kernel::RuntimeMassConstraint {
            name: "a".into(),
            expression: Atom::var(runtime[0]),
        }];
        let (actual, identity) = records::write_source(
            directory.path(),
            &input,
            &targets,
            &options,
            None,
            &runtime,
            &constraints,
        )
        .unwrap();
        let mut atoms = Atoms::default();
        let terms = input
            .terms()
            .iter()
            .map(|t| LegacyTerm {
                prefactor: atoms.push(t.prefactor()),
                powers: t.monomial_powers().iter().map(|a| atoms.push(a)).collect(),
                factors: t
                    .factors()
                    .iter()
                    .map(|f| LegacyFactor {
                        polynomial: atoms.push(f.polynomial()),
                        exponent: atoms.push(f.exponent()),
                        polynomial_role: f.role() == FactorRole::Polynomial,
                        semantics: f.semantics(),
                    })
                    .collect(),
            })
            .collect();
        let metadata = LegacySource {
            source_scope: None,
            source_identity: identity,
            parameters: 1,
            targets: 1,
            runtime_parameters: 1,
            runtime_mass_constraints: constraints
                .iter()
                .map(|c| (c.name.clone(), atoms.push(&c.expression)))
                .collect(),
            domain: 1,
            terms,
            options: (&options).into(),
        };
        let symbols = input
            .parameters()
            .iter()
            .copied()
            .chain([input.regulator()])
            .chain(targets)
            .chain(runtime)
            .collect();
        let expected = codec::write(
            directory.path(),
            "source",
            "source",
            &metadata,
            atoms,
            symbols,
        )
        .unwrap();
        assert_eq!(actual, expected);
        let context = records::read_source(directory.path(), &actual).unwrap();
        assert_eq!(context.input, input);
        assert_eq!(context.runtime_parameters, runtime);
        assert_eq!(context.targets, targets);
    }
}
