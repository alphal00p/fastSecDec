//! Gaussian numerator moments using native U/F and exact source derivatives.
//!
//! No external Gram inverse or separate tensor algebra is introduced. For a
//! numerator monomial of degree m in scalar products, append those scalar
//! products to the native inverse-propagator family as auxiliary source rows.
//! Differentiate m times and set the auxiliary parameters to zero. Integrating
//! the Schwinger radial scale first shifts Gamma(A-LD/2) to Gamma(A-LD/2-m).
//!
//! A numerator given as a polynomial in affine forms, such as `A * B^4 * C^4`,
//! can instead use those forms as source rows; see [`NumeratorSources`]. The
//! route taken is reported in [`NumeratorReport`]. Independent source moments
//! are computed in parallel only on a rayon pool the caller runs in; term
//! order and content do not depend on it.

mod factored;
mod moments;

use std::collections::BTreeSet;

use feynkit_graph::IntegralFamily;
use serde::{Deserialize, Serialize};
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    id::Pattern,
    symbol,
    transcendental::TranscendentalFunctions,
};

use crate::{
    Error, Result,
    input::{GraphIntegral, default_algebra_settings},
};

pub use factored::NumeratorFallback;

use super::scalar::validated_gamma;
use super::{
    FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
    PolynomialFactor,
};

/// Gaussian source rows used to parameterize a loop-dependent scalar numerator.
///
/// All choices give exactly the same projective density; they differ in the
/// number of terms and in the form of the polynomial numerator factors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumeratorSources {
    /// [`Self::Factors`] when the numerator is a polynomial in affine forms of
    /// which at least one is not a bare scalar product, and
    /// [`Self::ScalarProducts`] otherwise. The reason for using scalar
    /// products is reported in [`NumeratorReport::fallback`].
    #[default]
    Auto,
    /// Use the distinct affine forms of a numerator written as a polynomial in
    /// affine scalar-product forms, for example `A * B^4 * C^4` with
    /// `A = (k0+k1+k3-k4)^2 - m^2`, as source rows: one mixed derivative per
    /// product of forms. A numerator without that structure is an error
    /// ([`Error::UnfactoredNumerator`]) instead of an implicit fallback.
    Factors,
    /// One source row per family scalar product and one term per monomial of
    /// the expanded numerator (the historical representation).
    ScalarProducts,
}

/// The source rows actually used for a numerator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumeratorRoute {
    /// No loop-dependent numerator, or a scaleless or vanishing integral: no
    /// Gaussian sources.
    None,
    /// Affine forms as source rows.
    Factors,
    /// Family scalar products as source rows.
    ScalarProducts,
}

/// How a numerator was parameterized; independent of the thread count.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NumeratorReport {
    pub requested: NumeratorSources,
    pub route: NumeratorRoute,
    /// Why [`NumeratorRoute::ScalarProducts`] was used although factored
    /// sources were requested automatically.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<NumeratorFallback>,
    /// Gaussian source rows appended to the family.
    pub source_rows: usize,
    /// Source moments: products of forms or scalar-product monomials.
    pub moments: usize,
    /// Factored moments whose numerator factor was expanded in the Feynman
    /// parameters; the others contribute one term per monomial in the jets.
    #[serde(default)]
    pub expanded_moments: usize,
    /// Terms of the parameterized integrand.
    pub terms: usize,
}

impl NumeratorReport {
    fn new(requested: NumeratorSources, route: NumeratorRoute) -> Self {
        Self {
            requested,
            route,
            fallback: None,
            source_rows: 0,
            moments: 0,
            expanded_moments: 0,
            terms: 0,
        }
    }

    /// A fallback that a caller should surface: the numerator was not a
    /// polynomial in affine forms, or exceeded a bound. Bare scalar-product
    /// numerators use the same source rows on both routes.
    pub fn notable_fallback(&self) -> Option<NumeratorFallback> {
        self.fallback
            .filter(|reason| *reason != NumeratorFallback::BareScalarProducts)
    }
}

/// A factored numerator factor is expanded in the Feynman parameters only when
/// the expansion has at most this many monomials before merging; otherwise
/// each monomial in the jets becomes its own term. Jet terms keep products of
/// jets factored, which also keeps compiled kernels smaller: on one sector of
/// a five-loop rank-2 master (P29, sector 1776) generation took 14 s with two
/// jet terms, 31 s with the expanded factor and 20 s with the unexpanded sum.
const EXPAND_LIMIT: u128 = 64;

#[cfg(test)]
thread_local! {
    static EXPAND_LIMIT_OVERRIDE: std::cell::Cell<Option<u128>> = const { std::cell::Cell::new(None) };
}

fn expand_limit() -> u128 {
    #[cfg(test)]
    if let Some(limit) = EXPAND_LIMIT_OVERRIDE.with(std::cell::Cell::get) {
        return limit;
    }
    EXPAND_LIMIT
}

/// Fewer items than this are always mapped sequentially.
const PARALLEL_MINIMUM: usize = 8;

/// Map in order on the rayon pool the caller runs in, or sequentially outside
/// one. A library call therefore never starts rayon's global pool, which uses
/// every visible core; callers bound parallelism by installing their own pool.
pub(super) fn ordered_map<T, R, F>(items: Vec<T>, map: F) -> Vec<R>
where
    T: Send,
    R: Send,
    F: Fn(T) -> R + Sync + Send,
{
    if items.len() < PARALLEL_MINIMUM || rayon::current_thread_index().is_none() {
        items.into_iter().map(map).collect()
    } else {
        use rayon::prelude::*;
        items.into_par_iter().map(map).collect()
    }
}

impl ParametricIntegrand {
    /// Contract the native graph numerator and integrate its loop momenta by
    /// Gaussian source differentiation, retaining exact regulator dependence.
    pub fn from_graph(
        integral: &GraphIntegral,
        parameters: Vec<Symbol>,
        regulator: Symbol,
        dimension: Atom,
    ) -> Result<Self> {
        let numerator =
            integral.scalar_numerator(&default_algebra_settings())? * integral.measure_multiplier();
        Self::from_family(
            integral.family(),
            integral.powers(),
            numerator,
            parameters,
            regulator,
            dimension,
        )
    }

    /// Parameterize a native propagator family with strictly positive powers.
    ///
    /// `weighted_numerator` is the already contracted scalar numerator in the
    /// family's scalar-product notation, including every projector, graph
    /// factor and extra measure multiplier exactly once. This method adds only
    /// the normalized Minkowski loop measure `prod(d^D k / (i*pi^(D/2)))` and
    /// its Feynman-parameter Gamma/sign factors. It does not inspect a graph,
    /// perform partial fractions or apply another overall weight.
    ///
    /// For a term returned by [`IntegralFamily::partial_fraction`], callers can
    /// use [`IntegralFamily::sector`] to retain the positive-power denominators,
    /// keep their powers in the same order, and multiply the native coefficient
    /// and any negative-power denominator factors into `weighted_numerator`.
    /// Branches with different parameter lists must be parameterized separately.
    /// No loop shift or removal of a scaleless term is inferred from a projection.
    ///
    /// Parameters must be fresh, distinct symbols, separate from the regulator,
    /// weighted numerator and dimension. Native Symanzik construction validates
    /// collisions with the family expressions. A concrete tensor dimension
    /// cannot be changed; symbolic tensor dimensions are replaced literally.
    /// If propagator or kinematic coefficients themselves depend on that native
    /// dimension symbol, specialize the family first before changing dimension;
    /// this entry rejects leaving a stale native dimension in U or F.
    ///
    /// The default [`NumeratorSources`] are used; see
    /// [`Self::from_family_with_sources`].
    pub fn from_family(
        family: &IntegralFamily,
        powers: &[u32],
        weighted_numerator: Atom,
        parameters: Vec<Symbol>,
        regulator: Symbol,
        dimension: Atom,
    ) -> Result<Self> {
        Self::from_family_with_sources(
            family,
            powers,
            weighted_numerator,
            parameters,
            regulator,
            dimension,
            NumeratorSources::default(),
        )
        .map(|(integrand, _)| integrand)
    }

    /// [`Self::from_family`] with an explicit choice of Gaussian source rows,
    /// also reporting the route taken.
    pub fn from_family_with_sources(
        family: &IntegralFamily,
        powers: &[u32],
        weighted_numerator: Atom,
        parameters: Vec<Symbol>,
        regulator: Symbol,
        dimension: Atom,
        sources: NumeratorSources,
    ) -> Result<(Self, NumeratorReport)> {
        parameterize_family_with_sources(
            family,
            powers,
            weighted_numerator,
            parameters,
            regulator,
            dimension,
            sources,
        )
    }
}

fn parameterize_family_with_sources(
    family: &IntegralFamily,
    powers: &[u32],
    numerator: Atom,
    parameters: Vec<Symbol>,
    regulator: Symbol,
    dimension: Atom,
    sources: NumeratorSources,
) -> Result<(ParametricIntegrand, NumeratorReport)> {
    let uf = validate_family_input(
        family,
        powers,
        &numerator,
        &parameters,
        regulator,
        &dimension,
    )?;
    parameterize_validated_family(
        family, powers, numerator, parameters, regulator, dimension, uf, sources,
    )
}

pub(super) fn validate_family_input(
    family: &IntegralFamily,
    powers: &[u32],
    numerator: &Atom,
    parameters: &[Symbol],
    regulator: Symbol,
    dimension: &Atom,
) -> Result<(Atom, Atom)> {
    if powers.len() != family.denominators().len() || powers.contains(&0) {
        return Err(super::ParametricError::Invalid(
            "family parameterization requires one strictly positive power per denominator".into(),
        )
        .into());
    }
    if parameters.len() != family.denominators().len() {
        return Err(Error::ParameterCount {
            expected: family.denominators().len(),
            actual: parameters.len(),
        });
    }
    // Reuse common parameter/regulator admission before native Gaussian work;
    // native symanzik separately owns collisions with the family expressions.
    ParametricIntegrand::new(
        parameters.to_vec(),
        regulator,
        ParametricDomain::ProjectiveSimplex,
        vec![],
    )?;
    let native_dimension = family.kinematics().dimension().to_symbolic();
    let parameter_atoms = parameters.iter().map(|s| Atom::var(*s)).collect::<Vec<_>>();
    if let Some(parameter) = parameter_atoms.iter().find(|p| {
        numerator.contains(p.as_view())
            || dimension.contains(p.as_view())
            || native_dimension.contains(p.as_view())
    }) {
        return Err(Error::ParameterCollision(parameter.clone()));
    }
    if !matches!(native_dimension.as_view(), AtomView::Var(_)) && native_dimension != dimension {
        return Err(Error::ConcreteDimensionMismatch);
    }
    let (u, f) = family.symanzik(&parameter_atoms)?;
    if matches!(native_dimension.as_view(), AtomView::Var(_))
        && native_dimension != dimension
        && (u.contains(native_dimension.as_view()) || f.contains(native_dimension.as_view()))
    {
        return Err(super::ParametricError::Invalid(
            "specialize dimension-dependent propagator/kinematic coefficients before changing the native tensor dimension".into(),
        )
        .into());
    }
    if u.is_zero() {
        return Err(Error::SingularLoopForm);
    }
    Ok((u, f))
}

/// Expand the numerator in the family scalar products and reject coefficients
/// that still depend on loop momenta.
fn scalar_product_polynomial(
    numerator: &Atom,
    family: &IntegralFamily,
) -> Result<
    symbolica::poly::polynomial::MultivariatePolynomial<symbolica::domains::atom::AtomField, u32>,
> {
    let basis = family.scalar_products();
    let polynomial = numerator.to_polynomial_in_vars::<u32>(basis);
    for term in &polynomial {
        if basis.iter().any(|p| term.coefficient.contains(p.as_view()))
            || family
                .loop_momenta()
                .iter()
                .any(|p| term.coefficient.contains(p.as_view()))
            || term
                .coefficient
                .get_all_symbols(true)
                .contains(&feynkit_graph::symbols::loop_momentum())
        {
            return Err(Error::NonPolynomialNumerator);
        }
    }
    Ok(polynomial)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn parameterize_validated_family(
    family: &IntegralFamily,
    powers: &[u32],
    numerator: Atom,
    parameters: Vec<Symbol>,
    regulator: Symbol,
    dimension: Atom,
    (u, f): (Atom, Atom),
    numerator_sources: NumeratorSources,
) -> Result<(ParametricIntegrand, NumeratorReport)> {
    let native_dimension = family.kinematics().dimension().to_symbolic();
    let parameter_atoms = parameters.iter().map(|s| Atom::var(*s)).collect::<Vec<_>>();
    let replace_dimension = |expression: Atom| match native_dimension.as_view() {
        AtomView::Var(_) => expression
            .replace(Pattern::Literal(native_dimension.clone()))
            .with(Pattern::Literal(dimension.clone())),
        _ => expression,
    };
    let basis = family.scalar_products();
    let excluded = dimension
        .get_all_symbols(true)
        .into_iter()
        .chain(native_dimension.get_all_symbols(true))
        .chain([regulator])
        .collect::<Vec<_>>();
    let empty = |report: NumeratorReport| -> Result<(ParametricIntegrand, NumeratorReport)> {
        Ok((
            ParametricIntegrand::new(
                parameters.clone(),
                regulator,
                ParametricDomain::ProjectiveSimplex,
                vec![],
            )?,
            report,
        ))
    };
    // Choose the source rows first: the factored route never expands the
    // numerator in scalar products. Bare scalar-product forms keep the
    // historical route automatically, since its source rows are the same.
    let mut report = NumeratorReport::new(numerator_sources, NumeratorRoute::ScalarProducts);
    let factored = match numerator_sources {
        NumeratorSources::ScalarProducts => None,
        NumeratorSources::Auto | NumeratorSources::Factors => {
            match factored::factor(&numerator, family, &excluded) {
                // Loop independent: the scalar route below has no sources.
                Ok(factored) if factored.forms.is_empty() => None,
                Ok(factored) if factored.bare && numerator_sources == NumeratorSources::Auto => {
                    report.fallback = Some(NumeratorFallback::BareScalarProducts);
                    None
                }
                Ok(factored) => Some(factored),
                Err(reason) => {
                    // A numerator that is not polynomial in the scalar products
                    // is rejected as such on either route.
                    scalar_product_polynomial(&numerator, family)?;
                    if numerator_sources == NumeratorSources::Factors {
                        return Err(Error::UnfactoredNumerator(reason));
                    }
                    report.fallback = Some(reason);
                    None
                }
            }
        }
    };
    let polynomial = match &factored {
        Some(_) => None,
        None => Some(scalar_product_polynomial(&numerator, family)?),
    };
    // The native scaling certificate detects a vanishing dimensionally
    // regulated integral without treating F=0 alone as a proof.
    let vanishing = match (&factored, &polynomial) {
        (Some(factored), _) => factored.products.is_empty(),
        (None, Some(polynomial)) => polynomial.nterms() == 0,
        (None, None) => unreachable!("one numerator representation is selected"),
    };
    if vanishing || family.scaleless_scaling(&parameter_atoms)?.is_some() {
        report.route = NumeratorRoute::None;
        report.fallback = None;
        return empty(report);
    }
    let total_power = powers.iter().map(|p| u64::from(*p)).sum::<u64>();
    let loops = family.loop_momenta().len();
    let beta = (Atom::num(total_power) - &dimension * Atom::num(loops) / 2).expand();
    let base_u = (&beta - &dimension / 2).expand();
    let normalization = Atom::num(if total_power % 2 == 0 { 1 } else { -1 })
        / powers
            .iter()
            .map(|power| Atom::num(*power).gamma())
            .product::<Atom>();
    if let Some(polynomial) = &polynomial
        && polynomial
            .into_iter()
            .all(|term| term.exponents.iter().all(|power| *power == 0))
    {
        let coefficient = replace_dimension(numerator);
        report.route = NumeratorRoute::None;
        report.fallback = None;
        report.terms = 1;
        return Ok((
            ParametricIntegrand::new(
                parameters,
                regulator,
                ParametricDomain::ProjectiveSimplex,
                vec![ParametricTerm::new(
                    normalization * validated_gamma(beta.clone())? * coefficient,
                    powers.iter().map(|p| Atom::num(p - 1)).collect(),
                    vec![
                        PolynomialFactor::new(u, base_u, FactorRole::Singularity)
                            .with_semantics(FactorSemantics::Positive),
                        PolynomialFactor::new(f, -beta, FactorRole::Singularity)
                            .with_semantics(FactorSemantics::Causal),
                    ],
                )],
            )?,
            report,
        ));
    }
    let mut occupied = numerator
        .get_all_symbols(true)
        .into_iter()
        .collect::<BTreeSet<_>>();
    occupied.extend(parameters.iter().copied());
    occupied.extend(dimension.get_all_symbols(true));
    occupied.insert(regulator);
    for denominator in family.denominators() {
        occupied.extend(denominator.get_all_symbols(true));
    }
    // Each product of affine forms, or each scalar-product monomial, is one
    // source moment: (loop-independent coefficient, source orders).
    let (rows, products): (Vec<Atom>, Vec<(Atom, Vec<u32>)>) = match (&factored, polynomial) {
        (Some(_), _) => {
            let factored = factored.as_ref().expect("selected factored numerator");
            report.route = NumeratorRoute::Factors;
            (factored.forms.clone(), factored.products.clone())
        }
        (None, Some(polynomial)) => (
            basis.to_vec(),
            (&polynomial)
                .into_iter()
                .map(|monomial| (monomial.coefficient.clone(), monomial.exponents.to_vec()))
                .collect(),
        ),
        (None, None) => unreachable!("one numerator representation is selected"),
    };
    drop(factored);
    report.source_rows = rows.len();
    report.moments = products.len();
    let mut next = 0usize;
    let sources = rows
        .iter()
        .map(|_| {
            loop {
                let name = symbol!(&format!("fastsecdec::gaussian_source_{next}"));
                next += 1;
                if occupied.insert(name) {
                    break name;
                }
            }
        })
        .collect::<Vec<_>>();
    let extended_family = IntegralFamily::new(
        family.loop_momenta().to_vec(),
        family.external_momenta().to_vec(),
        family.denominators().iter().chain(&rows).cloned().collect(),
        family.kinematics(),
    )?;
    let extended_parameters = parameter_atoms
        .iter()
        .cloned()
        .chain(sources.iter().map(|s| Atom::var(*s)))
        .collect::<Vec<_>>();
    let (source_u, source_f) = extended_family.symanzik(&extended_parameters)?;
    let density_powers = powers.iter().map(|p| Atom::num(p - 1)).collect::<Vec<_>>();
    // Check every Gamma argument in term order before any derivative: at an
    // exact pole the polynomial can vanish, but dropping Gamma(pole)*0 would
    // lose a finite limit. The first offending term reports the error.
    let gammas = products
        .iter()
        .map(|(_, orders)| {
            let order = orders.iter().map(|p| u64::from(*p)).sum::<u64>();
            validated_gamma((&beta - Atom::num(order)).expand()).map(|gamma| (order, gamma))
        })
        .collect::<Result<Vec<_>>>()?;
    let kernel = (report.route == NumeratorRoute::Factors).then(|| {
        let zero_products = basis
            .iter()
            .map(|p| {
                symbolica::id::Replacement::new(
                    Pattern::Literal(p.clone()),
                    Pattern::Literal(Atom::Zero),
                )
            })
            .collect::<Vec<_>>();
        let constants = extended_family
            .denominators()
            .iter()
            .map(|row| row.replace_multiple(&zero_products).expand())
            .collect::<Vec<_>>();
        factored::SourceKernel::new(
            source_u.clone(),
            source_f.clone(),
            &parameter_atoms,
            &sources,
            &constants,
            &u,
        )
    });
    let namespace = factored::placeholder_namespace(&occupied);
    let singular = |u_exponent: Atom| {
        [
            PolynomialFactor::new(u.clone(), u_exponent.expand(), FactorRole::Singularity)
                .with_semantics(FactorSemantics::Positive),
            PolynomialFactor::new(f.clone(), -&beta, FactorRole::Singularity)
                .with_semantics(FactorSemantics::Causal),
        ]
    };
    // Moments are independent; a pool the caller runs in bounds the threads.
    // Ordered collection keeps the sequential term order.
    let moments = ordered_map(
        products.into_iter().zip(gammas).collect::<Vec<_>>(),
        |((coefficient, orders), (order, gamma))| -> (Vec<ParametricTerm>, bool) {
            let a = &base_u - Atom::num(order);
            let b = -&beta + Atom::num(order);
            let prefactor = &normalization * gamma * replace_dimension(coefficient);
            let u_exponent = &base_u - Atom::num(2 * order);
            let Some(kernel) = &kernel else {
                let numerator_polynomial =
                    moments::source_moment(&source_u, &source_f, &sources, &orders, &a, &b);
                if numerator_polynomial.is_zero() {
                    return (Vec::new(), false);
                }
                let [u_factor, f_factor] = singular(u_exponent);
                return (
                    vec![ParametricTerm::new(
                        prefactor,
                        density_powers.clone(),
                        vec![
                            u_factor,
                            f_factor,
                            PolynomialFactor::new(
                                numerator_polynomial,
                                Atom::one(),
                                FactorRole::Polynomial,
                            ),
                        ],
                    )],
                    false,
                );
            };
            let moment = kernel.moment(&orders, &a, &b, &namespace);
            let u_exponent = u_exponent + Atom::num(moment.u_shift);
            if moment.monomials.is_empty() {
                return (Vec::new(), false);
            }
            if moment.expansion_bound() <= expand_limit() {
                // Tiny: one numerator factor, expanded in the parameters.
                let numerator_polynomial = moment.polynomial().expand();
                if numerator_polynomial.is_zero() {
                    return (Vec::new(), true);
                }
                let [u_factor, f_factor] = singular(u_exponent);
                return (
                    vec![ParametricTerm::new(
                        prefactor,
                        density_powers.clone(),
                        vec![
                            u_factor,
                            f_factor,
                            PolynomialFactor::new(
                                numerator_polynomial,
                                Atom::one(),
                                FactorRole::Polynomial,
                            ),
                        ],
                    )],
                    true,
                );
            }
            // Large: one term per jet monomial, with the regulator-dependent
            // coefficient in the prefactor and every jet its own factor. Powers
            // of the native U are absorbed into its exponent.
            let terms = moment
                .monomials
                .iter()
                .map(|(jet_coefficient, exponents)| {
                    let mut u_power = 0u32;
                    let mut jets = Vec::new();
                    for (jet, exponent) in moment.jets.iter().zip(exponents) {
                        if *exponent == 0 {
                            continue;
                        }
                        if jet.native_u {
                            u_power += exponent;
                        } else {
                            jets.push(PolynomialFactor::new(
                                jet.value.clone(),
                                Atom::num(*exponent),
                                FactorRole::Polynomial,
                            ));
                        }
                    }
                    let [u_factor, f_factor] = singular(&u_exponent + Atom::num(u_power));
                    ParametricTerm::new(
                        &prefactor * jet_coefficient,
                        density_powers.clone(),
                        [u_factor, f_factor].into_iter().chain(jets).collect(),
                    )
                })
                .collect();
            (terms, false)
        },
    );
    report.expanded_moments = moments.iter().filter(|(_, expanded)| *expanded).count();
    let terms = moments
        .into_iter()
        .flat_map(|(terms, _)| terms)
        .collect::<Vec<_>>();
    let integrand = ParametricIntegrand::new(
        parameters,
        regulator,
        ParametricDomain::ProjectiveSimplex,
        terms,
    )?;
    report.terms = integrand.terms().len();
    Ok((integrand, report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use feynkit_kinematics::Kinematics;
    use symbolica::parse;

    /// The historical test entry with the default numerator sources.
    fn parameterize_family(
        family: &IntegralFamily,
        powers: &[u32],
        numerator: Atom,
        parameters: Vec<Symbol>,
        regulator: Symbol,
        dimension: Atom,
    ) -> Result<ParametricIntegrand> {
        super::parameterize_family_with_sources(
            family,
            powers,
            numerator,
            parameters,
            regulator,
            dimension,
            NumeratorSources::default(),
        )
        .map(|(integrand, _)| integrand)
    }

    fn massive_tadpole(s: Atom) -> (IntegralFamily, Atom, Atom) {
        let (k, p) = (parse!("k"), parse!("p"));
        let kin = Kinematics::in_dimension(&parse!("D"))
            .unwrap()
            .with_momenta([k.clone(), p.clone()])
            .unwrap()
            .with_mass_squared(&p, s)
            .unwrap();
        let kk = kin.scalar_product(&k, &k).unwrap();
        let kp = kin.scalar_product(&k, &p).unwrap();
        let family = IntegralFamily::new(vec![k], vec![p], vec![&kk - parse!("M")], &kin).unwrap();
        (family, kk, kp)
    }

    fn finite_tadpole(family: &IntegralFamily, numerator: Atom, power: u32) -> Atom {
        let integrand = parameterize_family(
            family,
            &[power],
            numerator,
            vec![symbol!("x")],
            symbol!("eps"),
            parse!("2-2*eps"),
        )
        .unwrap();
        integrand
            .density()
            .replace(symbol!("x"))
            .with(1)
            .series(symbol!("eps"), 0, 0)
            .unwrap()
            .to_atom()
            .together()
    }

    #[test]
    fn massive_tadpole_numerator_cancels_a_propagator_with_raised_powers() {
        let (family, kk, _) = massive_tadpole(parse!("s"));
        let value = finite_tadpole(&family, &kk - parse!("M"), 5);
        assert!((value - parse!("1/(3*M^3)")).together().is_zero());
        let squared = finite_tadpole(&family, kk.pow(2), 5);
        assert!((squared - parse!("-1/(12*M^2)")).together().is_zero());
    }

    #[test]
    fn rank_two_and_four_match_independent_lorentz_gaussian_moments() {
        let (family, _, kp) = massive_tadpole(parse!("s"));
        // In D=2, <(k.p)^2>=p^2<k^2>/2 and
        // <(k.p)^4>=3p^4<k^4>/(2*4), using the normalized Minkowski measure.
        let rank2 = finite_tadpole(&family, kp.pow(2), 5);
        assert!(
            (&rank2 - parse!("s/(24*M^3)")).together().is_zero(),
            "rank2={rank2}"
        );
        let rank4 = finite_tadpole(&family, kp.pow(4), 5);
        assert!(
            (&rank4 - parse!("-s^2/(32*M^2)")).together().is_zero(),
            "rank4={rank4}"
        );
    }

    #[test]
    fn gaussian_sources_do_not_invert_a_degenerate_external_gram_matrix() {
        let (family, _, kp) = massive_tadpole(Atom::Zero);
        assert!(finite_tadpole(&family, kp.pow(2), 5).is_zero());
    }

    #[test]
    fn nonpolynomial_loop_numerator_is_rejected_explicitly() {
        let (family, kk, _) = massive_tadpole(parse!("s"));
        let result = parameterize_family(
            &family,
            &[5],
            kk.pow(-1),
            vec![symbol!("x")],
            symbol!("eps"),
            parse!("2-2*eps"),
        );
        assert!(matches!(result, Err(Error::NonPolynomialNumerator)));
    }

    #[test]
    fn native_scaleless_certificate_handles_polynomial_numerators() {
        let (family, kk, _) = massive_tadpole(parse!("s"));
        let massless = IntegralFamily::new(
            family.loop_momenta().to_vec(),
            family.external_momenta().to_vec(),
            vec![kk.clone()],
            family.kinematics(),
        )
        .unwrap();
        let integrand = parameterize_family(
            &massless,
            &[2],
            kk.pow(2),
            vec![symbol!("x")],
            symbol!("eps"),
            parse!("4-2*eps"),
        )
        .unwrap();
        assert!(integrand.terms().is_empty());
    }

    #[test]
    fn exact_dimension_does_not_evaluate_a_removable_gamma_pole() {
        let (family, _, kp) = massive_tadpole(parse!("s"));
        let result = parameterize_family(
            &family,
            &[5],
            kp.pow(4),
            vec![symbol!("x")],
            symbol!("eps"),
            Atom::num(2),
        );
        assert!(matches!(result, Err(Error::UnregulatedGammaPole(_))));
    }

    /// Laurent coefficients of a density at a fixed exact point.
    fn density_series(integrand: &ParametricIntegrand, point: &[(&str, Atom)]) -> Atom {
        let mut density = integrand.density();
        for (name, value) in point {
            density = density.replace(symbol!(name)).with(value.clone());
        }
        density
            .series(symbol!("eps"), 0, 1)
            .unwrap()
            .to_atom()
            .expand()
    }

    #[test]
    fn factored_sources_match_scalar_products_with_an_external_momentum() {
        let (family, kk, kp) = massive_tadpole(parse!("s"));
        // ((k+p)^2-M)^2 has a k.p component, so the source F is not U times
        // an affine form and the generic U/F jet kernel is used.
        let form = &kk + Atom::num(2) * &kp + parse!("s") - parse!("M");
        let numerator = form.pow(2) * parse!("c");
        let route = |sources| {
            super::parameterize_family_with_sources(
                &family,
                &[5],
                numerator.clone(),
                vec![symbol!("x")],
                symbol!("eps"),
                parse!("2-2*eps"),
                sources,
            )
            .unwrap()
        };
        let (factored, report) = route(NumeratorSources::Auto);
        let (scalar, _) = route(NumeratorSources::ScalarProducts);
        assert_eq!(report.route, NumeratorRoute::Factors);
        assert_eq!((report.source_rows, report.moments), (1, 1));
        assert_eq!(factored.terms().len(), 1);
        assert!(scalar.terms().len() > 1);
        let point = [("x", Atom::one())];
        assert!(
            (density_series(&factored, &point) - density_series(&scalar, &point))
                .together()
                .is_zero()
        );
    }

    #[test]
    fn factored_sources_match_scalar_products_for_a_two_loop_vacuum_sum() {
        let (k, l) = (parse!("k"), parse!("l"));
        let kin = Kinematics::in_dimension(&parse!("D"))
            .unwrap()
            .with_momenta([k.clone(), l.clone()])
            .unwrap();
        let kk = kin.scalar_product(&k, &k).unwrap();
        let ll = kin.scalar_product(&l, &l).unwrap();
        let kl = kin.scalar_product(&k, &l).unwrap();
        let family = IntegralFamily::new(
            vec![k, l],
            vec![],
            vec![&kk - Atom::num(1), &ll - Atom::num(2)],
            &kin,
        )
        .unwrap();
        // A sum of products of affine forms with different total orders and a
        // constant; without external momenta F = U * L uses the vacuum kernel.
        let a = &kk + Atom::num(2) * &kl + &ll - Atom::num(3);
        let b = &kk - Atom::num(5);
        let numerator = Atom::num(7) * a.pow(2) * &b - &a + Atom::num(11);
        let route = |sources| {
            super::parameterize_family_with_sources(
                &family,
                &[3, 3],
                numerator.clone(),
                vec![symbol!("x"), symbol!("y")],
                symbol!("eps"),
                parse!("4-2*eps"),
                sources,
            )
            .unwrap()
        };
        let (factored, report) = route(NumeratorSources::Factors);
        let (scalar, scalar_report) = route(NumeratorSources::ScalarProducts);
        assert_eq!(report.route, NumeratorRoute::Factors);
        assert_eq!(scalar_report.route, NumeratorRoute::ScalarProducts);
        assert_eq!(scalar_report.fallback, None);
        // 7 a^2 b and the affine part -a + 11, which is one form.
        assert_eq!((report.source_rows, report.moments), (3, 2));
        assert_eq!(factored.terms().len(), report.terms);
        assert!(scalar.terms().len() > 3);
        let point = [("x", parse!("1/3")), ("y", parse!("2/3"))];
        assert!(
            (density_series(&factored, &point) - density_series(&scalar, &point))
                .together()
                .is_zero()
        );
    }

    #[test]
    fn bare_scalar_product_factors_keep_the_monomial_route() {
        let (family, kk, kp) = massive_tadpole(parse!("s"));
        let numerator = kk.pow(2) * kp.pow(2);
        let route = |sources| {
            super::parameterize_family_with_sources(
                &family,
                &[5],
                numerator.clone(),
                vec![symbol!("x")],
                symbol!("eps"),
                parse!("2-2*eps"),
                sources,
            )
            .unwrap()
        };
        let (automatic, report) = route(NumeratorSources::Auto);
        assert_eq!(automatic, route(NumeratorSources::ScalarProducts).0);
        assert_eq!(report.route, NumeratorRoute::ScalarProducts);
        assert_eq!(report.fallback, Some(NumeratorFallback::BareScalarProducts));
        assert_eq!(report.notable_fallback(), None);
        // Requested explicitly, bare forms use the factored kernel exactly.
        let (factored, report) = route(NumeratorSources::Factors);
        assert_eq!(report.route, NumeratorRoute::Factors);
        assert_eq!((report.source_rows, report.moments), (2, 1));
        let point = [("x", parse!("2/7"))];
        assert!(
            (density_series(&factored, &point) - density_series(&automatic, &point))
                .together()
                .is_zero()
        );
    }

    fn tadpole_route(
        numerator: &Atom,
        sources: NumeratorSources,
    ) -> Result<(ParametricIntegrand, NumeratorReport)> {
        let (family, _, _) = massive_tadpole(parse!("s"));
        super::parameterize_family_with_sources(
            &family,
            &[5],
            numerator.clone(),
            vec![symbol!("x")],
            symbol!("eps"),
            parse!("2-2*eps"),
            sources,
        )
    }

    /// Without affine structure the automatic choice falls back visibly and an
    /// explicit request for factored sources is an error.
    #[test]
    fn requested_factored_sources_never_fall_back_silently() {
        let (_, kk, kp) = massive_tadpole(parse!("s"));
        // Three distinct forms in two scalar products.
        let numerator = (&kk + Atom::num(1)) * (&kp + Atom::num(2)) * (&kk + &kp + Atom::num(3));
        let (automatic, report) = tadpole_route(&numerator, NumeratorSources::Auto).unwrap();
        assert_eq!(report.route, NumeratorRoute::ScalarProducts);
        assert_eq!(
            report.notable_fallback(),
            Some(NumeratorFallback::TooManyForms)
        );
        assert_eq!(report.terms, automatic.terms().len());
        assert_eq!(
            automatic,
            tadpole_route(&numerator, NumeratorSources::ScalarProducts)
                .unwrap()
                .0
        );
        assert!(matches!(
            tadpole_route(&numerator, NumeratorSources::Factors),
            Err(Error::UnfactoredNumerator(NumeratorFallback::TooManyForms))
        ));
        // A non-polynomial numerator is rejected as such on every route.
        for sources in [NumeratorSources::Auto, NumeratorSources::Factors] {
            assert!(matches!(
                tadpole_route(&(&kk + Atom::num(1)).pow(Atom::num(-1)), sources),
                Err(Error::NonPolynomialNumerator)
            ));
        }
        // Regulator-dependent coefficients and regulator-dependent constants of a
        // form are moved out of the forms instead of forcing a fallback.
        let eps = parse!("eps");
        for numerator in [
            &eps * (&kp + Atom::num(1)).pow(2) + (Atom::num(4) - Atom::num(2) * &eps) * &kk,
            (&kk + Atom::num(2) * &kp - &eps).pow(2),
        ] {
            let (factored, report) = tadpole_route(&numerator, NumeratorSources::Factors).unwrap();
            let (scalar, _) = tadpole_route(&numerator, NumeratorSources::ScalarProducts).unwrap();
            assert_eq!(report.route, NumeratorRoute::Factors);
            let point = [("x", parse!("3/5"))];
            assert!(
                (density_series(&factored, &point) - density_series(&scalar, &point))
                    .together()
                    .is_zero()
            );
        }
    }

    /// Both factored representations, one expanded numerator factor per
    /// moment and one term per jet monomial, give the scalar-product density.
    #[test]
    fn expanded_and_jet_term_representations_match_scalar_products() {
        let (k, l, p) = (parse!("k"), parse!("l"), parse!("p"));
        let kin = Kinematics::in_dimension(&parse!("D"))
            .unwrap()
            .with_momenta([k.clone(), l.clone(), p.clone()])
            .unwrap()
            .with_mass_squared(&p, parse!("s"))
            .unwrap();
        let sp = |a: &Atom, b: &Atom| kin.scalar_product(a, b).unwrap();
        let (kk, ll, kl, kp, lp, pp) = (
            sp(&k, &k),
            sp(&l, &l),
            sp(&k, &l),
            sp(&k, &p),
            sp(&l, &p),
            sp(&p, &p),
        );
        let m = parse!("m");
        let d3 = &kk + &ll + &pp + Atom::num(2) * (&kl + &kp + &lp) - &m;
        let vacuum_kin = Kinematics::in_dimension(&parse!("D"))
            .unwrap()
            .with_momenta([k.clone(), l.clone()])
            .unwrap();
        let vsp = |a: &Atom, b: &Atom| vacuum_kin.scalar_product(a, b).unwrap();
        let (vkk, vll, vkl) = (vsp(&k, &k), vsp(&l, &l), vsp(&k, &l));
        let cases = [
            (
                IntegralFamily::new(
                    vec![k.clone(), l.clone()],
                    vec![p.clone()],
                    vec![&kk - &m, &ll - &m, d3],
                    &kin,
                )
                .unwrap(),
                (&kk + Atom::num(2) * &kp - Atom::num(3)).pow(2) * (&lp + Atom::num(2)),
                vec![1, 1, 2],
            ),
            (
                IntegralFamily::new(
                    vec![k.clone(), l.clone()],
                    vec![],
                    vec![&vkk - &m, &vll - &m, &vkk - Atom::num(2) * &vkl + &vll - &m],
                    &vacuum_kin,
                )
                .unwrap(),
                (&vkk - Atom::num(2) * &vkl - Atom::num(5)).pow(2) * (&vll + Atom::num(3))
                    - Atom::num(2) * (&vkl + Atom::num(1)),
                vec![2, 1, 1],
            ),
        ];
        let point = [
            ("x0", parse!("1/5")),
            ("x1", parse!("3/10")),
            ("x2", parse!("1/2")),
            ("s", parse!("-3/7")),
            ("m", parse!("2/3")),
        ];
        for (family, numerator, powers) in cases {
            let route = |sources, limit| {
                EXPAND_LIMIT_OVERRIDE.with(|cell| cell.set(limit));
                let result = super::parameterize_family_with_sources(
                    &family,
                    &powers,
                    numerator.clone(),
                    vec![symbol!("x0"), symbol!("x1"), symbol!("x2")],
                    symbol!("eps"),
                    parse!("4-2*eps"),
                    sources,
                )
                .unwrap();
                EXPAND_LIMIT_OVERRIDE.with(|cell| cell.set(None));
                result
            };
            let (scalar, _) = route(NumeratorSources::ScalarProducts, None);
            let (expanded, expanded_report) = route(NumeratorSources::Factors, Some(u128::MAX));
            let (jets, jet_report) = route(NumeratorSources::Factors, Some(0));
            assert_eq!(expanded_report.expanded_moments, expanded_report.moments);
            assert_eq!(expanded_report.terms, expanded_report.moments);
            assert_eq!(jet_report.expanded_moments, 0);
            assert!(jet_report.terms > jet_report.moments);
            // Jet terms carry regulator-free polynomial factors.
            for term in jets.terms() {
                for factor in term.factors() {
                    assert!(!factor.polynomial().contains_symbol(symbol!("eps")));
                }
            }
            let reference = density_series(&scalar, &point);
            for factored in [&expanded, &jets] {
                assert!(
                    (density_series(factored, &point) - &reference)
                        .together()
                        .is_zero()
                );
            }
        }
    }

    #[test]
    fn shifted_loop_routing_reproduces_the_same_scalar_integral() {
        let (family, kk, kp) = massive_tadpole(parse!("s"));
        let shifted = IntegralFamily::new(
            family.loop_momenta().to_vec(),
            family.external_momenta().to_vec(),
            vec![&kk + 2 * &kp + parse!("s-M")],
            family.kinematics(),
        )
        .unwrap();
        // q=k+p: k^2=q^2-2q.p+p^2, and the odd tadpole moment vanishes.
        let value = finite_tadpole(&shifted, kk, 5);
        assert!(
            (value - parse!("1/(12*M^3)-s/(4*M^4)"))
                .together()
                .is_zero()
        );
    }

    #[test]
    fn two_loop_mixed_moment_matches_the_product_of_lorentz_moments() {
        let (k, l) = (parse!("k"), parse!("l"));
        let kin = Kinematics::in_dimension(&parse!("D"))
            .unwrap()
            .with_momenta([k.clone(), l.clone()])
            .unwrap();
        let kk = kin.scalar_product(&k, &k).unwrap();
        let ll = kin.scalar_product(&l, &l).unwrap();
        let kl = kin.scalar_product(&k, &l).unwrap();
        let family = IntegralFamily::new(vec![k, l], vec![], vec![kk - 1, ll - 1], &kin).unwrap();
        let result = parameterize_family(
            &family,
            &[4, 4],
            kl.pow(2),
            vec![symbol!("x"), symbol!("y")],
            symbol!("eps"),
            parse!("2-2*eps"),
        )
        .unwrap();
        let finite = result
            .density()
            .replace(symbol!("y"))
            .with(parse!("1-x"))
            .series(symbol!("eps"), 0, 0)
            .unwrap()
            .to_atom()
            .cancel()
            .expand();
        // Symbolica supplies the exact polynomial antiderivative. Each one-loop
        // k^2 moment equals -1/6, and the mixed contraction divides by D=2.
        let primitive = finite
            .to_polynomial_in_vars::<u32>(symbol!("x"))
            .integrate(0)
            .flatten(false);
        let integral =
            primitive.replace(symbol!("x")).with(1) - primitive.replace(symbol!("x")).with(0);
        assert!((integral - parse!("1/72")).together().is_zero());
    }
}
