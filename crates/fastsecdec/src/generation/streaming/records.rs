//! Native source/chart/formula record representations. No alternate algebra.
use super::codec::{self, Atoms, RecordRef, StreamingError, invalid};
use super::input::{Input, Term};
use crate::{
    generation::{
        CoefficientExpansionOptions, GenerationMode, GenerationOptions, SubtractionStrategy,
        mapping::MappedTerm,
        numerical_dual::{
            DualFactor, DualTerm,
            subtraction::{Coordinate, Recipe, Request},
        },
    },
    parametric::{FactorSemantics, ParametricIntegrand},
};
use fastsecdec_sectors::{DecompositionOptions, SectorMap};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
use symbolica::atom::{AliasedAtom, AtomCore, Symbol};
mod prepared;
pub(super) use prepared::{PreparedData, read_prepared, write_prepared};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Options {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_sectors: Option<Vec<usize>>,
    mode: GenerationMode,
    #[serde(
        default,
        skip_serializing_if = "crate::contour::ContourJacobian::is_symbolic"
    )]
    contour_jacobian: crate::contour::ContourJacobian,
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
            source_sectors: x.source_sectors.clone(),
            mode: x.mode,
            contour_jacobian: x.contour_jacobian,
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
            source_sectors: x.source_sectors,
            mode: x.mode,
            contour_jacobian: x.contour_jacobian,
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
struct Source {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_scope: Option<crate::generation::GenerationSourceScope>,
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
    pub source_scope: Option<crate::generation::GenerationSourceScope>,
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
    source_scope: Option<&crate::generation::GenerationSourceScope>,
    runtime: &[Symbol],
    constraints: &[crate::kernel::RuntimeMassConstraint],
) -> Result<(RecordRef, String), StreamingError> {
    let mut atoms = Atoms::default();
    let Input {
        parameters,
        domain,
        terms,
    } = Input::encode(input, &mut atoms);
    let source = Source {
        source_scope: source_scope.cloned(),
        source_identity: crate::generation::source_identity(input, runtime, constraints)?,
        parameters,
        targets: targets.len(),
        runtime_parameters: runtime.len(),
        runtime_mass_constraints: constraints
            .iter()
            .map(|c| (c.name.clone(), atoms.push(&c.expression)))
            .collect(),
        domain,
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
    if let Some(scope) = &source.source_scope {
        scope.validate(scope.selection().source_sectors().len())?;
        if scope.chart_source_sectors() != scope.selection().source_sectors() {
            return Err(invalid(
                "source context must retain the complete requested subset",
            ));
        }
        let mut requested = source.options.source_sectors.clone().unwrap_or_default();
        requested.sort_unstable();
        if requested != scope.selection().source_sectors() {
            return Err(invalid(
                "source selection differs from recorded generation options",
            ));
        }
    }
    if symbols.len() != source.parameters + 1 + source.targets + source.runtime_parameters {
        return Err(invalid("source symbol layout mismatch"));
    }
    let input = Input {
        parameters: source.parameters,
        domain: source.domain,
        terms: source.terms,
    }
    .decode(&atoms, &symbols[..source.parameters + 1])?;
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
        source_scope: source.source_scope,
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
    #[serde(
        default,
        skip_serializing_if = "crate::contour::ContourJacobian::is_symbolic"
    )]
    contour_jacobian: crate::contour::ContourJacobian,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    jacobians: Vec<(usize, JacobianPlan)>,
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
    fn native(
        self,
        atoms: &Atoms,
        definitions: std::sync::Arc<crate::contour::ContourDefinitions>,
    ) -> Result<crate::contour::ContourMetadata, StreamingError> {
        let restore = |indices: Vec<usize>| {
            indices
                .into_iter()
                .map(|i| atoms.take(i))
                .collect::<Result<Vec<_>, _>>()
        };
        Ok(crate::contour::ContourMetadata {
            definitions,
            causal_polynomial: atoms.take(self.causal_polynomial)?,
            positive_polynomials: restore(self.positive_polynomials)?,
            images: restore(self.images)?,
            ratios: restore(self.ratios)?,
            jacobian: atoms.take(self.jacobian)?,
            validation_faces: self.validation_faces,
        })
    }
}
// These indices refer to the same native StateMap/Atom table as the chart.
// The optional semantic source is consumed at compilation, not saved as a
// second evaluator or reconstructed from process-local symbol names.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct JacobianPlan {
    parameters: Vec<usize>,
    images: Vec<usize>,
    jacobian: usize,
}
impl JacobianPlan {
    fn save(
        plan: &crate::contour::ContourJacobianPlan,
        contour: &Contour,
        atoms: &mut Atoms,
    ) -> Self {
        Self {
            parameters: plan
                .parameters
                .iter()
                .map(|p| atoms.push(&symbolica::atom::Atom::var(*p)))
                .collect(),
            images: contour.images.clone(),
            jacobian: contour.jacobian,
        }
    }
    fn native(self, atoms: &Atoms) -> Result<crate::contour::ContourJacobianPlan, StreamingError> {
        let parameters = self
            .parameters
            .into_iter()
            .map(|i| {
                atoms
                    .take(i)?
                    .as_var_view()
                    .map(|v| v.get_symbol())
                    .ok_or_else(|| invalid("Jacobian plan coordinate is not a native symbol"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let images = self
            .images
            .into_iter()
            .map(|i| atoms.take(i))
            .collect::<Result<Vec<_>, _>>()?;
        if !(1..=6).contains(&parameters.len())
            || images.len() != parameters.len()
            || parameters
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != parameters.len()
        {
            return Err(invalid("invalid native dual Jacobian plan dimension"));
        }
        Ok(crate::contour::ContourJacobianPlan {
            parameters,
            images,
            jacobian: atoms.take(self.jacobian)?,
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
    let contour = data
        .contour
        .as_ref()
        .map(|metadata| Contour::save(metadata, &mut atoms));
    let jacobians =
        data.program
            .jacobians
            .iter()
            .map(|(index, plan)| {
                let metadata = data.contour.as_ref().ok_or_else(|| {
                    invalid("dual Jacobian source requires native contour metadata")
                })?;
                if plan.images != metadata.images() || plan.jacobian != *metadata.jacobian() {
                    return Err(invalid(
                        "dual Jacobian plan differs from its native contour chart",
                    ));
                }
                Ok((
                    *index,
                    JacobianPlan::save(plan, contour.as_ref().unwrap(), &mut atoms),
                ))
            })
            .collect::<Result<Vec<_>, StreamingError>>()?;
    codec::write_with_program(
        root,
        &format!("chart-{}", data.index),
        "chart",
        &Chart {
            contour_jacobian: data.program.contour_jacobian,
            jacobians,
            index: data.index,
            source_id: data.source_id.clone(),
            map: (&data.map).into(),
            mapped,
            deferred,
            contour,
        },
        atoms,
        vec![],
        &data.program,
    )
}
pub(super) fn read_chart(root: &Path, reference: &RecordRef) -> Result<ChartData, StreamingError> {
    let (chart, atoms, _, mut program): (Chart, _, _, _) =
        codec::read_with_program(root, reference, "chart")?;
    program.contour_jacobian = chart.contour_jacobian;
    program.jacobians = chart
        .jacobians
        .into_iter()
        .map(|(index, plan)| Ok((index, std::sync::Arc::new(plan.native(&atoms)?))))
        .collect::<Result<_, StreamingError>>()?;
    if !program.jacobians.is_empty() && program.contour_jacobian.is_symbolic() {
        return Err(invalid("symbolic chart contains a dual Jacobian plan"));
    }
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
    let definitions = program.contour_definitions()?;
    let contour = chart
        .contour
        .map(|metadata| metadata.native(&atoms, definitions))
        .transpose()?;
    let needs_plan = program.contour_jacobian == crate::contour::ContourJacobian::Dual
        && contour
            .as_ref()
            .is_some_and(|metadata| !metadata.images().is_empty());
    if needs_plan {
        let [(index, plan)] = program.jacobians.as_slice() else {
            return Err(invalid(
                "dual contour chart must retain exactly one Jacobian plan",
            ));
        };
        let metadata = contour.as_ref().unwrap();
        // Saved single-chart program owners are record-local, while chart.index
        // retains the original source index used by the streaming job.
        if *index != 0 || plan.images != metadata.images() || plan.jacobian != *metadata.jacobian()
        {
            return Err(invalid(
                "dual Jacobian plan differs from its native contour chart",
            ));
        }
    } else if !program.jacobians.is_empty() {
        return Err(invalid(
            "unexpected dual Jacobian plan for an undeformed or zero-dimensional chart",
        ));
    }
    Ok(ChartData {
        program,
        index: chart.index,
        source_id: chart.source_id,
        map: chart.map.native()?,
        mapped,
        deferred,
        contour,
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
