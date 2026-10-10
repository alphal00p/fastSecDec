use super::*;
use crate::generation::mapping::PreparedTerm;
use symbolica::{parse, symbol};

fn term(f: Atom, u: Atom) -> PreparedTerm {
    PreparedTerm {
        powers: vec![parse!("-1+eps"), Atom::Zero],
        prefactor: parse!("2+eps"),
        residuals: vec![
            (f, Atom::Zero, FactorSemantics::Causal),
            (u, Atom::Zero, FactorSemantics::Positive),
            (parse!("1+x+2*y"), Atom::one(), FactorSemantics::Generic),
        ],
    }
}

#[test]
fn source_witness_preserves_zero_exponent_branches_and_recipe() {
    let parameters = [symbol!("x"), symbol!("y")];
    let prepare = |f, u, recipe| {
        SourceWitness::new(&parameters, &[term(f, u)], recipe)
            .unwrap()
            .unwrap()
            .prepare(&parameters)
            .unwrap()
    };
    for recipe in [ProgramRecipe::FixedV1, ProgramRecipe::DynamicPolynomialV1] {
        let source = prepare(parse!("1-x-y"), parse!("1+x+y"), recipe);
        for target in [
            prepare(parse!("2-2*x-2*y"), parse!("1+x+y"), recipe),
            prepare(parse!("1-x-y"), parse!("1+x+2*y"), recipe),
            prepare(
                parse!("1-x-y"),
                parse!("1+x+y"),
                if recipe == ProgramRecipe::FixedV1 {
                    ProgramRecipe::DynamicPolynomialV1
                } else {
                    ProgramRecipe::FixedV1
                },
            ),
        ] {
            assert_eq!(source.density, target.density);
            assert!(source.equivalent_to(&target).unwrap().is_none());
        }
    }
}

#[test]
fn source_witness_proves_the_complete_density_permutation() {
    let parameters = [symbol!("x"), symbol!("y")];
    let original = term(parse!("1-x-2*y"), parse!("1+x+y"));
    let swap = |atom: &Atom| {
        atom.replace_multiple([
            Replacement::new(Pattern::Literal(parse!("x")), Pattern::Literal(parse!("y"))),
            Replacement::new(Pattern::Literal(parse!("y")), Pattern::Literal(parse!("x"))),
        ])
    };
    let permuted = PreparedTerm {
        powers: vec![original.powers[1].clone(), original.powers[0].clone()],
        prefactor: swap(&original.prefactor),
        residuals: original
            .residuals
            .iter()
            .map(|(factor, power, role)| (swap(factor), swap(power), *role))
            .collect(),
    };
    for recipe in [ProgramRecipe::FixedV1, ProgramRecipe::DynamicPolynomialV1] {
        let source = SourceWitness::new(
            &parameters,
            &[PreparedTerm {
                powers: original.powers.clone(),
                prefactor: original.prefactor.clone(),
                residuals: original.residuals.clone(),
            }],
            recipe,
        )
        .unwrap()
        .unwrap()
        .prepare(&parameters)
        .unwrap();
        let target = SourceWitness::new(
            &parameters,
            &[PreparedTerm {
                powers: permuted.powers.clone(),
                prefactor: permuted.prefactor.clone(),
                residuals: permuted.residuals.clone(),
            }],
            recipe,
        )
        .unwrap()
        .unwrap()
        .prepare(&parameters)
        .unwrap();
        assert_eq!(source.equivalent_to(&target).unwrap(), Some(vec![1, 0]));
        let mut changed = target;
        changed.density *= Atom::num(2);
        // Even an otherwise identical candidate graph cannot waive the native
        // exact proof of prefactors, regulator powers and numerator factors.
        assert!(source.equivalent_to(&changed).unwrap().is_none());
    }
}

#[test]
fn canceled_source_terms_keep_the_declared_map_witness() {
    let parameters = [symbol!("x"), symbol!("y")];
    let first = term(parse!("1-x-y"), parse!("1+x+y"));
    let mut second = term(parse!("1-x-y"), parse!("1+x+y"));
    second.prefactor = -second.prefactor;
    let witness = SourceWitness::new(
        &parameters,
        &[first, second],
        ProgramRecipe::DynamicPolynomialV1,
    )
    .unwrap()
    .unwrap();
    assert!(witness.density.expand_num().is_zero());
    assert_eq!(witness.causal, parse!("1-x-y"));
    assert_eq!(witness.positive, vec![parse!("1+x+y")]);
}

#[test]
fn fractional_causal_and_principal_factors_do_not_become_one_plain_power() {
    let x = symbol!("x");
    let f = parse!("1-2*x");
    let half = Atom::num((1, 2));
    assert!((f.pow(&half) * f.pow(-&half)).is_one());
    let source = PreparedTerm {
        powers: vec![Atom::Zero],
        prefactor: Atom::one(),
        residuals: vec![
            (f.clone(), half.clone(), FactorSemantics::Causal),
            (f.clone(), -half.clone(), FactorSemantics::Generic),
        ],
    };
    let target = PreparedTerm {
        powers: vec![Atom::Zero],
        prefactor: Atom::one(),
        residuals: vec![
            (f.clone(), Atom::Zero, FactorSemantics::Causal),
            (f.clone(), half.clone(), FactorSemantics::Generic),
            (f, -half, FactorSemantics::Generic),
        ],
    };
    let prepare = |term| {
        SourceWitness::new(&[x], &[term], ProgramRecipe::DynamicPolynomialV1)
            .unwrap()
            .unwrap()
            .prepare(&[x])
            .unwrap()
    };
    let a = prepare(source);
    let b = prepare(target);
    assert_eq!(a.witness, b.witness); // Same F, U and map construction.
    assert_ne!(a.density, b.density); // Distinct branch continuation survives.
    assert!(a.equivalent_to(&b).unwrap().is_none());
}
