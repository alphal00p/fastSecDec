//! Gaussian numerator moments using native U/F and exact source derivatives.
//!
//! No external Gram inverse or separate tensor algebra is introduced. For a
//! numerator monomial of degree m in scalar products, append those scalar
//! products to the native inverse-propagator family as auxiliary source rows.
//! Differentiate m times and set the auxiliary parameters to zero. Integrating
//! the Schwinger radial scale first shifts Gamma(A-LD/2) to Gamma(A-LD/2-m).
//!
//! A numerator given as a product of powers of affine forms, such as
//! `A * B^4 * C^4`, can instead use those forms as source rows; see
//! [`NumeratorSources`]. Independent source moments are computed in parallel
//! on the caller's rayon pool; term order and content do not depend on it.

mod factored;
mod moments;

use std::collections::BTreeSet;

use feynkit_graph::IntegralFamily;
use rayon::prelude::*;
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

use super::scalar::validated_gamma;
use super::{
    FactorRole, FactorSemantics, ParametricDomain, ParametricIntegrand, ParametricTerm,
    PolynomialFactor,
};

/// Gaussian source rows used to parameterize a loop-dependent scalar numerator.
///
/// Both choices give exactly the same projective density; they differ in the
/// number of terms and in the form of each polynomial numerator factor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumeratorSources {
    /// Use the distinct affine factors of a numerator written as a sum of
    /// products of positive integer powers of affine scalar-product forms, for
    /// example `A * B^4 * C^4` with `A = (k0+k1+k3-k4)^2 - m^2`, as source
    /// rows: one term and one mixed derivative per product. Numerators without
    /// such a structure, or whose factors are all bare scalar products, use
    /// [`Self::ScalarProducts`] unchanged.
    #[default]
    Factors,
    /// One source row per family scalar product and one term per monomial of
    /// the expanded numerator (the historical representation).
    ScalarProducts,
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
    }

    /// [`Self::from_family`] with an explicit choice of Gaussian source rows.
    pub fn from_family_with_sources(
        family: &IntegralFamily,
        powers: &[u32],
        weighted_numerator: Atom,
        parameters: Vec<Symbol>,
        regulator: Symbol,
        dimension: Atom,
        sources: NumeratorSources,
    ) -> Result<Self> {
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
) -> Result<ParametricIntegrand> {
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
) -> Result<ParametricIntegrand> {
    let native_dimension = family.kinematics().dimension().to_symbolic();
    let parameter_atoms = parameters.iter().map(|s| Atom::var(*s)).collect::<Vec<_>>();
    let replace_dimension = |expression: Atom| match native_dimension.as_view() {
        AtomView::Var(_) => expression
            .replace(Pattern::Literal(native_dimension.clone()))
            .with(Pattern::Literal(dimension.clone())),
        _ => expression,
    };
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
    // The native scaling certificate detects a vanishing dimensionally
    // regulated integral without treating F=0 alone as a proof.
    if polynomial.nterms() == 0 || family.scaleless_scaling(&parameter_atoms)?.is_some() {
        return Ok(ParametricIntegrand::new(
            parameters,
            regulator,
            ParametricDomain::ProjectiveSimplex,
            vec![],
        )?);
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
    if (&polynomial)
        .into_iter()
        .all(|term| term.exponents.iter().all(|power| *power == 0))
    {
        let coefficient = replace_dimension(numerator);
        return Ok(ParametricIntegrand::new(
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
        )?);
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
    let excluded = dimension
        .get_all_symbols(true)
        .into_iter()
        .chain(native_dimension.get_all_symbols(true))
        .chain([regulator])
        .collect::<Vec<_>>();
    let factors = match numerator_sources {
        NumeratorSources::Factors => factored::factor(&numerator, family, &excluded),
        NumeratorSources::ScalarProducts => None,
    };
    // Each product of affine forms, or each scalar-product monomial, becomes
    // one term: (loop-independent coefficient, source orders).
    let factored = factors.is_some();
    let (rows, products): (Vec<Atom>, Vec<(Atom, Vec<u32>)>) = match factors {
        Some(factors) => (factors.forms, factors.products),
        None => (
            basis.to_vec(),
            (&polynomial)
                .into_iter()
                .map(|monomial| (monomial.coefficient.clone(), monomial.exponents.to_vec()))
                .collect(),
        ),
    };
    drop(polynomial);
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
    let kernel = factored.then(|| {
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
        )
    });
    let namespace = factored::placeholder_namespace(&occupied);
    // Moments are independent; the caller's rayon pool bounds the threads.
    // Indexed collection keeps the sequential term order.
    let terms = products
        .par_iter()
        .zip(gammas.into_par_iter())
        .map(|((coefficient, orders), (order, gamma))| {
            let a = &base_u - Atom::num(order);
            let b = -&beta + Atom::num(order);
            let (numerator_polynomial, u_shift) = match &kernel {
                Some(kernel) => kernel.moment(orders, &a, &b, &namespace),
                None => (
                    moments::source_moment(&source_u, &source_f, &sources, orders, &a, &b),
                    0,
                ),
            };
            if numerator_polynomial.is_zero() {
                return None;
            }
            let coefficient = replace_dimension(coefficient.clone());
            Some(ParametricTerm::new(
                &normalization * gamma * coefficient,
                density_powers.clone(),
                vec![
                    PolynomialFactor::new(
                        u.clone(),
                        if u_shift == 0 {
                            (&base_u - Atom::num(2 * order)).expand()
                        } else {
                            (&base_u - Atom::num(2 * order) + Atom::num(u_shift)).expand()
                        },
                        FactorRole::Singularity,
                    )
                    .with_semantics(FactorSemantics::Positive),
                    PolynomialFactor::new(f.clone(), -&beta, FactorRole::Singularity)
                        .with_semantics(FactorSemantics::Causal),
                    PolynomialFactor::new(
                        numerator_polynomial,
                        Atom::one(),
                        FactorRole::Polynomial,
                    ),
                ],
            ))
        })
        .collect::<Vec<_>>()
        .into_iter()
        .flatten()
        .collect();
    Ok(ParametricIntegrand::new(
        parameters,
        regulator,
        ParametricDomain::ProjectiveSimplex,
        terms,
    )?)
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
        let factored = route(NumeratorSources::Factors);
        let scalar = route(NumeratorSources::ScalarProducts);
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
        let factored = route(NumeratorSources::Factors);
        let scalar = route(NumeratorSources::ScalarProducts);
        assert_eq!(factored.terms().len(), 3);
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
        assert_eq!(
            route(NumeratorSources::Factors),
            route(NumeratorSources::ScalarProducts)
        );
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
