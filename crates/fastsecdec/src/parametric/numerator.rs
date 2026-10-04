//! Gaussian numerator moments using native U/F and exact source derivatives.
//!
//! No external Gram inverse or separate tensor algebra is introduced. For a
//! numerator monomial of degree m in scalar products, append those scalar
//! products to the native inverse-propagator family as auxiliary source rows.
//! Differentiate m times and set the auxiliary parameters to zero. Integrating
//! the Schwinger radial scale first shifts Gamma(A-LD/2) to Gamma(A-LD/2-m).

use std::collections::BTreeSet;

use feynkit_graph::IntegralFamily;
use symbolica::{
    atom::{Atom, AtomCore, AtomView, Symbol},
    id::{Pattern, Replacement},
    symbol,
    transcendental::TranscendentalFunctions,
};

use crate::{
    Error, Result,
    input::{GraphIntegral, default_algebra_settings},
};

use super::scalar::validated_gamma;
use super::{FactorRole, ParametricDomain, ParametricIntegrand, ParametricTerm, PolynomialFactor};

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
    pub fn from_family(
        family: &IntegralFamily,
        powers: &[u32],
        weighted_numerator: Atom,
        parameters: Vec<Symbol>,
        regulator: Symbol,
        dimension: Atom,
    ) -> Result<Self> {
        parameterize_family(
            family,
            powers,
            weighted_numerator,
            parameters,
            regulator,
            dimension,
        )
    }
}

fn parameterize_family(
    family: &IntegralFamily,
    powers: &[u32],
    numerator: Atom,
    parameters: Vec<Symbol>,
    regulator: Symbol,
    dimension: Atom,
) -> Result<ParametricIntegrand> {
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
        parameters.clone(),
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
    let replace_dimension = |expression: Atom| match native_dimension.as_view() {
        AtomView::Var(_) => expression
            .replace(Pattern::Literal(native_dimension.clone()))
            .with(Pattern::Literal(dimension.clone())),
        _ => expression,
    };
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
                    PolynomialFactor::new(u, base_u, FactorRole::Singularity),
                    PolynomialFactor::new(f, -beta, FactorRole::Singularity),
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
    let mut next = 0usize;
    let sources = basis
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
        family.denominators().iter().chain(basis).cloned().collect(),
        family.kinematics(),
    )?;
    let extended_parameters = parameter_atoms
        .iter()
        .cloned()
        .chain(sources.iter().map(|s| Atom::var(*s)))
        .collect::<Vec<_>>();
    let (source_u, source_f) = extended_family.symanzik(&extended_parameters)?;
    let derivatives = sources
        .iter()
        .map(|s| (source_u.derivative(*s), source_f.derivative(*s)))
        .collect::<Vec<_>>();
    let zero_sources = sources
        .iter()
        .map(|s| {
            Replacement::new(
                Pattern::Literal(Atom::var(*s)),
                Pattern::Literal(Atom::Zero),
            )
        })
        .collect::<Vec<_>>();
    let density_powers = powers.iter().map(|p| Atom::num(p - 1)).collect::<Vec<_>>();
    let mut terms = Vec::with_capacity(polynomial.nterms());
    for monomial in &polynomial {
        let order = monomial
            .exponents
            .iter()
            .map(|p| u64::from(*p))
            .sum::<u64>();
        // Check before evaluating derivatives: at an exact pole the polynomial
        // can vanish, but dropping Gamma(pole)*0 would lose a finite limit.
        let gamma = validated_gamma((&beta - Atom::num(order)).expand())?;
        let a = &base_u - Atom::num(order);
        let b = -&beta + Atom::num(order);
        let mut differentiated = Atom::one();
        let mut completed = 0u64;
        // ∂[U^(a-r) F^(b-r) P] = U^(a-r-1) F^(b-r-1)
        // * [UF ∂P + ((a-r)F ∂U + (b-r)U ∂F)P].
        for (index, count) in monomial.exponents.iter().enumerate() {
            for _ in 0..*count {
                let (du, df) = &derivatives[index];
                differentiated = &source_u * &source_f * differentiated.derivative(sources[index])
                    + ((&a - Atom::num(completed)) * &source_f * du
                        + (&b - Atom::num(completed)) * &source_u * df)
                        * differentiated;
                completed += 1;
            }
        }
        // Preserve factorization through differentiation and source elimination.
        let numerator_polynomial = differentiated.replace_multiple(&zero_sources);
        if numerator_polynomial.is_zero() {
            continue;
        }
        let coefficient = replace_dimension(monomial.coefficient.clone());
        terms.push(ParametricTerm::new(
            &normalization * gamma * coefficient,
            density_powers.clone(),
            vec![
                PolynomialFactor::new(
                    u.clone(),
                    (&base_u - Atom::num(2 * order)).expand(),
                    FactorRole::Singularity,
                ),
                PolynomialFactor::new(f.clone(), -&beta, FactorRole::Singularity),
                PolynomialFactor::new(numerator_polynomial, Atom::one(), FactorRole::Polynomial),
            ],
        ));
    }
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
