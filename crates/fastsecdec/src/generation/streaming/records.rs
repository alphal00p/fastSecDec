//! Native source/chart/formula record representations. No alternate algebra.
use super::codec::{self, Atoms, RecordRef, StreamingError, invalid};
use crate::{
    generation::{
        CoefficientExpansionOptions, GenerationMode, GenerationOptions, SubtractionStrategy,
        mapping::MappedTerm,
        numerical_dual::{
            DualFactor, DualTerm,
            subtraction::{Coordinate, Recipe, Request},
        },
    },
    parametric::{
        FactorRole, FactorSemantics, ParametricIntegrand, ParametricTerm, PolynomialFactor,
    },
};
use fastsecdec_sectors::{DecompositionOptions, ParametricDomain, SectorMap};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
use symbolica::atom::{AliasedAtom, AtomCore, Symbol};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Options {
    mode: GenerationMode,
    assume_no_threshold: bool,
    program_recipe: crate::kernel::indexed::ProgramRecipe,
    max_order: i32,
    max_support_pairs: usize,
    max_rays: usize,
    max_sectors: usize,
    max_subtractions_per_axis: usize,
    max_subtraction_terms: usize,
    subtraction: SubtractionStrategy,
    coefficient_expansion: CoefficientExpansionOptions,
}
impl From<&GenerationOptions> for Options {
    fn from(x: &GenerationOptions) -> Self {
        Self {
            mode: x.mode,
            assume_no_threshold: x.assume_no_threshold,
            program_recipe: x.program_recipe,
            max_order: x.max_order,
            max_support_pairs: x.decomposition.max_support_pairs,
            max_rays: x.decomposition.max_rays,
            max_sectors: x.decomposition.max_sectors,
            max_subtractions_per_axis: x.max_subtractions_per_axis,
            max_subtraction_terms: x.max_subtraction_terms,
            subtraction: x.subtraction,
            coefficient_expansion: x.coefficient_expansion.clone(),
        }
    }
}
impl From<Options> for GenerationOptions {
    fn from(x: Options) -> Self {
        Self {
            mode: x.mode,
            assume_no_threshold: x.assume_no_threshold,
            program_recipe: x.program_recipe,
            max_order: x.max_order,
            decomposition: DecompositionOptions {
                max_support_pairs: x.max_support_pairs,
                max_rays: x.max_rays,
                max_sectors: x.max_sectors,
            },
            max_subtractions_per_axis: x.max_subtractions_per_axis,
            max_subtraction_terms: x.max_subtraction_terms,
            subtraction: x.subtraction,
            coefficient_expansion: x.coefficient_expansion,
        }
    }
}
#[derive(Serialize, Deserialize)]
pub(super) struct Map {
    fixed_parameter: Option<usize>,
    exponent_matrix: Vec<Vec<String>>,
    determinant: String,
    jacobian_powers: Vec<String>,
    factor_valuations: Vec<Vec<String>>,
}
impl From<&SectorMap> for Map {
    fn from(x: &SectorMap) -> Self {
        let rows = |v: &[Vec<symbolica::domains::integer::Integer>]| {
            v.iter()
                .map(|r| r.iter().map(ToString::to_string).collect())
                .collect()
        };
        Self {
            fixed_parameter: x.fixed_parameter,
            exponent_matrix: rows(&x.exponent_matrix),
            determinant: x.determinant.to_string(),
            jacobian_powers: x.jacobian_powers.iter().map(ToString::to_string).collect(),
            factor_valuations: rows(&x.factor_valuations),
        }
    }
}
impl Map {
    pub fn native(self) -> Result<SectorMap, StreamingError> {
        fn row(
            v: Vec<String>,
        ) -> Result<Vec<symbolica::domains::integer::Integer>, StreamingError> {
            v.into_iter()
                .map(|s| {
                    s.parse()
                        .map_err(|_| invalid("invalid native geometry integer"))
                })
                .collect()
        }
        Ok(SectorMap {
            fixed_parameter: self.fixed_parameter,
            exponent_matrix: self
                .exponent_matrix
                .into_iter()
                .map(row)
                .collect::<Result<_, _>>()?,
            determinant: self
                .determinant
                .parse()
                .map_err(|_| invalid("invalid determinant"))?,
            jacobian_powers: row(self.jacobian_powers)?,
            factor_valuations: self
                .factor_valuations
                .into_iter()
                .map(row)
                .collect::<Result<_, _>>()?,
        })
    }
}
#[derive(Serialize, Deserialize)]
struct Factor {
    polynomial: usize,
    exponent: usize,
    polynomial_role: bool,
    #[serde(default)]
    semantics: FactorSemantics,
}
#[derive(Serialize, Deserialize)]
struct Term {
    prefactor: usize,
    powers: Vec<usize>,
    factors: Vec<Factor>,
}
#[derive(Serialize, Deserialize)]
struct Source {
    source_identity: String,
    parameters: usize,
    targets: usize,
    runtime_parameters: usize,
    runtime_mass_constraints: Vec<(String, usize)>,
    domain: u8,
    terms: Vec<Term>,
    options: Options,
}
pub(super) struct Context {
    pub source_identity: String,
    pub input: ParametricIntegrand,
    pub targets: Vec<Symbol>,
    pub options: GenerationOptions,
    pub runtime_parameters: Vec<Symbol>,
    pub runtime_mass_constraints: Vec<crate::kernel::RuntimeMassConstraint>,
}
pub(super) fn write_source(
    root: &Path,
    input: &ParametricIntegrand,
    targets: &[Symbol],
    options: &GenerationOptions,
    runtime: &[Symbol],
    constraints: &[crate::kernel::RuntimeMassConstraint],
) -> Result<(RecordRef, String), StreamingError> {
    let mut atoms = Atoms::default();
    let terms = input
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
        .collect();
    let source = Source {
        source_identity: crate::generation::source_identity(input, runtime, constraints)?,
        parameters: input.parameters().len(),
        targets: targets.len(),
        runtime_parameters: runtime.len(),
        runtime_mass_constraints: constraints
            .iter()
            .map(|c| (c.name.clone(), atoms.push(&c.expression)))
            .collect(),
        domain: match input.domain() {
            ParametricDomain::ProjectiveSimplex => 0,
            ParametricDomain::UnitCube => 1,
            ParametricDomain::PositiveOrthant => 2,
        },
        terms,
        options: options.into(),
    };
    // Physical identity uses the same native canonical representation as the
    // public geometry-free helper; staging bytes retain process-local symbols
    // and therefore identify only this recipe's serialized execution context.
    let symbols = input
        .parameters()
        .iter()
        .copied()
        .chain([input.regulator()])
        .chain(targets.iter().copied())
        .chain(runtime.iter().copied())
        .collect();
    let identity = source.source_identity.clone();
    let record = codec::write(root, "source", "source", &source, atoms, symbols)?;
    Ok((record, identity))
}
pub(super) fn read_source(root: &Path, reference: &RecordRef) -> Result<Context, StreamingError> {
    let (source, atoms, symbols): (Source, _, _) = codec::read(root, reference, "source")?;
    if symbols.len() != source.parameters + 1 + source.targets + source.runtime_parameters {
        return Err(invalid("source symbol layout mismatch"));
    }
    let domain = match source.domain {
        0 => ParametricDomain::ProjectiveSimplex,
        1 => ParametricDomain::UnitCube,
        2 => ParametricDomain::PositiveOrthant,
        _ => return Err(invalid("source domain")),
    };
    let terms = source
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
    let input = ParametricIntegrand::new(
        symbols[..source.parameters].to_vec(),
        symbols[source.parameters],
        domain,
        terms,
    )
    .map_err(super::super::GenerationError::from)?;
    let runtime_start = source.parameters + 1 + source.targets;
    let runtime_mass_constraints = source
        .runtime_mass_constraints
        .into_iter()
        .map(|(name, id)| {
            Ok(crate::kernel::RuntimeMassConstraint {
                name,
                expression: atoms.take(id)?,
            })
        })
        .collect::<Result<_, StreamingError>>()?;
    Ok(Context {
        source_identity: source.source_identity,
        input,
        targets: symbols[source.parameters + 1..runtime_start].to_vec(),
        options: source.options.into(),
        runtime_parameters: symbols[runtime_start..].to_vec(),
        runtime_mass_constraints,
    })
}

#[derive(Serialize, Deserialize)]
struct Mapped {
    powers: Vec<usize>,
    prefactor: usize,
    regular: usize,
}
#[derive(Serialize, Deserialize)]
struct DeferredFactor {
    polynomial: usize,
    exponent: usize,
    valuation: Vec<i32>,
}
#[derive(Serialize, Deserialize)]
struct Chart {
    index: usize,
    source_id: String,
    map: Map,
    mapped: Vec<Mapped>,
    deferred: Option<Vec<Vec<DeferredFactor>>>,
    #[serde(default)]
    contour: Option<Contour>,
}
#[derive(Serialize, Deserialize)]
struct Contour {
    causal_polynomial: usize,
    positive_polynomials: Vec<usize>,
    images: Vec<usize>,
    ratios: Vec<usize>,
    jacobian: usize,
    validation_faces: Vec<Vec<(usize, u8)>>,
}
impl Contour {
    fn save(metadata: &crate::contour::ContourMetadata, atoms: &mut Atoms) -> Self {
        Self {
            causal_polynomial: atoms.push(metadata.causal_polynomial()),
            positive_polynomials: metadata
                .positive_polynomials()
                .iter()
                .map(|a| atoms.push(a))
                .collect(),
            images: metadata.images().iter().map(|a| atoms.push(a)).collect(),
            ratios: metadata.ratios().iter().map(|a| atoms.push(a)).collect(),
            jacobian: atoms.push(metadata.jacobian()),
            validation_faces: metadata.validation_faces().to_vec(),
        }
    }
    fn native(self, atoms: &Atoms) -> Result<crate::contour::ContourMetadata, StreamingError> {
        let restore = |indices: Vec<usize>| {
            indices
                .into_iter()
                .map(|i| atoms.take(i))
                .collect::<Result<Vec<_>, _>>()
        };
        Ok(crate::contour::ContourMetadata {
            causal_polynomial: atoms.take(self.causal_polynomial)?,
            positive_polynomials: restore(self.positive_polynomials)?,
            images: restore(self.images)?,
            ratios: restore(self.ratios)?,
            jacobian: atoms.take(self.jacobian)?,
            validation_faces: self.validation_faces,
        })
    }
}
pub(super) struct ChartData {
    pub program: super::super::program::ProgramData,
    pub index: usize,
    pub source_id: String,
    pub map: SectorMap,
    pub mapped: Vec<MappedTerm>,
    pub deferred: Option<Vec<DualTerm>>,
    pub contour: Option<crate::contour::ContourMetadata>,
}
pub(super) fn write_chart(root: &Path, data: &ChartData) -> Result<RecordRef, StreamingError> {
    let mut atoms = Atoms::default();
    let mapped = data
        .mapped
        .iter()
        .map(|t| Mapped {
            powers: t.powers.iter().map(|a| atoms.push(a)).collect(),
            prefactor: atoms.push(&t.prefactor),
            regular: atoms.push(&t.regular),
        })
        .collect();
    let deferred = data.deferred.as_ref().map(|terms| {
        terms
            .iter()
            .map(|term| {
                term.factors
                    .iter()
                    .map(|f| DeferredFactor {
                        polynomial: atoms.push(&f.polynomial),
                        exponent: atoms.push(&f.exponent),
                        valuation: f.valuation.clone(),
                    })
                    .collect()
            })
            .collect()
    });
    codec::write_with_program(
        root,
        &format!("chart-{}", data.index),
        "chart",
        &Chart {
            index: data.index,
            source_id: data.source_id.clone(),
            map: (&data.map).into(),
            mapped,
            deferred,
            contour: data
                .contour
                .as_ref()
                .map(|metadata| Contour::save(metadata, &mut atoms)),
        },
        atoms,
        vec![],
        &data.program,
    )
}
pub(super) fn read_chart(root: &Path, reference: &RecordRef) -> Result<ChartData, StreamingError> {
    let (chart, atoms, _, program): (Chart, _, _, _) =
        codec::read_with_program(root, reference, "chart")?;
    let mapped = chart
        .mapped
        .into_iter()
        .map(|t| {
            Ok(MappedTerm {
                prefactor: atoms.take(t.prefactor)?,
                regular: atoms.take(t.regular)?,
                powers: t
                    .powers
                    .into_iter()
                    .map(|i| atoms.take(i))
                    .collect::<Result<_, _>>()?,
            })
        })
        .collect::<Result<_, StreamingError>>()?;
    let deferred = chart
        .deferred
        .map(|terms| {
            terms
                .into_iter()
                .map(|factors| {
                    Ok(DualTerm {
                        factors: factors
                            .into_iter()
                            .map(|f| {
                                Ok(DualFactor {
                                    polynomial: atoms.take(f.polynomial)?,
                                    exponent: atoms.take(f.exponent)?,
                                    valuation: f.valuation,
                                })
                            })
                            .collect::<Result<_, StreamingError>>()?,
                    })
                })
                .collect::<Result<_, StreamingError>>()
        })
        .transpose()?;
    Ok(ChartData {
        program,
        index: chart.index,
        source_id: chart.source_id,
        map: chart.map.native()?,
        mapped,
        deferred,
        contour: chart
            .contour
            .map(|metadata| metadata.native(&atoms))
            .transpose()?,
    })
}

#[derive(Serialize, Deserialize)]
struct Aliased {
    root: usize,
    aliases: Vec<(usize, usize)>,
}
#[derive(Serialize, Deserialize)]
struct JetRequest {
    term: usize,
    epsilon_order: usize,
    derivatives: Vec<usize>,
    coordinates: Vec<(u8, usize)>,
}
#[derive(Serialize, Deserialize)]
struct Formula {
    key: String,
    signature: Vec<Mapped>,
    coefficients: BTreeMap<i32, Aliased>,
    requests: Vec<JetRequest>,
}
pub(super) fn write_formula(
    root: &Path,
    key: &str,
    signature: &[MappedTerm],
    recipe: &Recipe,
) -> Result<RecordRef, StreamingError> {
    let mut atoms = Atoms::default();
    let coefficients = recipe
        .coefficients
        .iter()
        .map(|(order, value)| {
            let mut aliases = value.get_aliases().iter().collect::<Vec<_>>();
            aliases.sort_by_key(|(a, _)| a.to_canonical_string());
            (
                *order,
                Aliased {
                    root: atoms.push(value.get_root()),
                    aliases: aliases
                        .into_iter()
                        .map(|(a, b)| (atoms.push(a), atoms.push(b)))
                        .collect(),
                },
            )
        })
        .collect();
    let requests = recipe
        .requests
        .iter()
        .map(|r| JetRequest {
            term: r.term,
            epsilon_order: r.epsilon_order,
            derivatives: r.derivatives.clone(),
            coordinates: r
                .coordinates
                .iter()
                .map(|c| match c {
                    Coordinate::Variable(i) => (0, *i),
                    Coordinate::Zero => (1, 0),
                    Coordinate::One => (2, 0),
                })
                .collect(),
        })
        .collect();
    let signature = signature
        .iter()
        .map(|t| Mapped {
            powers: t.powers.iter().map(|a| atoms.push(a)).collect(),
            prefactor: atoms.push(&t.prefactor),
            regular: atoms.push(&t.regular),
        })
        .collect();
    codec::write(
        root,
        &format!("formula-{key}"),
        "formula",
        &Formula {
            key: key.into(),
            signature,
            coefficients,
            requests,
        },
        atoms,
        recipe.requests.iter().map(|r| r.placeholder).collect(),
    )
}
pub(super) fn read_formula(
    root: &Path,
    reference: &RecordRef,
    key: &str,
) -> Result<(Recipe, Vec<MappedTerm>), StreamingError> {
    let (formula, atoms, symbols): (Formula, _, _) = codec::read(root, reference, "formula")?;
    if formula.key != key || formula.requests.len() != symbols.len() {
        return Err(invalid("formula signature or request layout mismatch"));
    }
    let coefficients = formula
        .coefficients
        .into_iter()
        .map(|(order, value)| {
            let mut out = AliasedAtom::from(atoms.take(value.root)?);
            for (a, b) in value.aliases {
                out.register_alias(atoms.take(a)?, atoms.take(b)?);
            }
            Ok((order, out))
        })
        .collect::<Result<_, StreamingError>>()?;
    let requests = formula
        .requests
        .into_iter()
        .zip(symbols)
        .map(|(r, placeholder)| {
            Ok(Request {
                placeholder,
                term: r.term,
                epsilon_order: r.epsilon_order,
                derivatives: r.derivatives,
                coordinates: r
                    .coordinates
                    .into_iter()
                    .map(|(tag, i)| match (tag, i) {
                        (0, i) => Ok(Coordinate::Variable(i)),
                        (1, 0) => Ok(Coordinate::Zero),
                        (2, 0) => Ok(Coordinate::One),
                        _ => Err(invalid("formula coordinate")),
                    })
                    .collect::<Result<_, _>>()?,
            })
        })
        .collect::<Result<_, StreamingError>>()?;
    let signature = formula
        .signature
        .into_iter()
        .map(|t| {
            Ok(MappedTerm {
                powers: t
                    .powers
                    .into_iter()
                    .map(|i| atoms.take(i))
                    .collect::<Result<_, _>>()?,
                prefactor: atoms.take(t.prefactor)?,
                regular: atoms.take(t.regular)?,
            })
        })
        .collect::<Result<_, StreamingError>>()?;
    Ok((
        Recipe {
            coefficients,
            requests,
        },
        signature,
    ))
}
