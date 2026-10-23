//! Native tensor interfaces survive graph weights, projection, and momentum routing.

use std::sync::Arc;

use fastsecdec::{
    Atom, AtomCore, Error, FeynmanDiagram, Kinematics, Model,
    input::{NumeratorContraction, contract_numerator, default_algebra_settings},
};
use feynkit_graph::symbols;
use idenso::tensor::SymbolicTensor;
use spenso::structure::representation::{Minkowski, RepName};
use symbolica::parse;

fn bubble() -> FeynmanDiagram {
    let model = Arc::new(
        Model::from_json(include_str!("../../../examples/models/massless_phi3.json")).unwrap(),
    );
    FeynmanDiagram::from_dot(model, include_str!("../../../examples/graphs/bubble.dot")).unwrap()
}

#[test]
fn native_tensor_factors_and_routing_preserve_exact_projected_numerator() {
    let k = symbols::loop_momentum().call(0);
    let q = symbols::momentum().call(2);
    let p = symbols::external_momentum().call(1);
    let kin = Kinematics::in_dimension(&parse!("D"))
        .unwrap()
        .with_mass_squared(&p, parse!("s"))
        .unwrap();
    let rep = Minkowski {}.new_rep(4);
    let qmu = rep.vector(q.as_view(), [Atom::num(1)]);
    let pmu = rep.vector(p.as_view(), [Atom::num(1)]);
    let diagram = bubble()
        .with_numerator(SymbolicTensor::infer(&qmu + &pmu).unwrap())
        .unwrap()
        .with_projector(
            SymbolicTensor::infer(Atom::num(5) * (Atom::num(2) * qmu - Atom::num(3) * pmu))
                .unwrap(),
        )
        .with_numerator_prefactor(SymbolicTensor::infer(Atom::num(11)).unwrap())
        .with_overall_factor(Atom::num(7));
    let expected = Atom::num(385)
        * (Atom::num(2) * kin.scalar_product(&k, &k).unwrap()
            - kin.scalar_product(&k, &p).unwrap()
            - Atom::num(3) * parse!("s"));
    for mode in [NumeratorContraction::Dots, NumeratorContraction::Full] {
        let actual = contract_numerator(&diagram, &kin, &mode.algebra_settings()).unwrap();
        assert!((actual - &expected).expand().is_zero(), "{mode:?}");
    }
}

#[test]
fn uncontracted_native_ports_remain_an_explicit_error() {
    let q = symbols::momentum().call(2);
    let vector = Minkowski {}.new_rep(4).vector(q.as_view(), [Atom::num(1)]);
    let diagram = bubble()
        .with_numerator(SymbolicTensor::infer(vector).unwrap())
        .unwrap();
    let kin = Kinematics::in_dimension(&parse!("D")).unwrap();
    assert!(matches!(
        contract_numerator(&diagram, &kin, &default_algebra_settings()),
        Err(Error::FreeTensorIndices)
    ));
}

#[test]
fn contracted_lorentz_trace_uses_the_requested_symbolic_dimension() {
    let metric =
        SymbolicTensor::infer(parse!("spenso::g(spenso::mink(4,mu),spenso::mink(4,nu))")).unwrap();
    let diagram = bubble()
        .with_numerator(metric.clone())
        .unwrap()
        .with_projector(metric)
        .with_numerator_prefactor(SymbolicTensor::infer(Atom::num(3)).unwrap())
        .with_overall_factor(Atom::num(5));
    let kin = Kinematics::in_dimension(&parse!("D")).unwrap();
    let actual = contract_numerator(&diagram, &kin, &default_algebra_settings()).unwrap();
    assert!((actual - parse!("15*D")).expand().is_zero());
}

#[test]
fn dimension_promotion_contracts_newly_equal_native_ports() {
    let kin = Kinematics::in_dimension(&parse!("D")).unwrap();
    let q = symbols::momentum().call(2);
    let fixed = Minkowski {}.new_rep(4).vector(q.as_view(), [Atom::num(1)]);
    let symbolic = Minkowski {}
        .new_rep(kin.dimension())
        .vector(q.as_view(), [Atom::num(1)]);
    let diagram = bubble()
        .with_numerator(SymbolicTensor::infer(fixed).unwrap())
        .unwrap()
        .with_projector(SymbolicTensor::infer(symbolic).unwrap());
    let actual = contract_numerator(&diagram, &kin, &default_algebra_settings()).unwrap();
    let k = symbols::loop_momentum().call(0);
    let expected = kin.scalar_product(&k, &k).unwrap();
    assert!((actual - expected).expand().is_zero());
}
