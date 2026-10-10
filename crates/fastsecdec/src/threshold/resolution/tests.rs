use super::*;
use std::sync::Arc;
use symbolica::{atom::Atom, domains::rational::Rational, parse, symbol};

struct Fixture {
    ring: Arc<Ring>,
    source: Arc<Chart>,
    adapted: Arc<Chart>,
    target: Arc<Chart>,
}
impl Fixture {
    fn new(budget: &mut Budget) -> Self {
        let (p, x, y, z, a, b, c, u, v, t, w) = symbol!(
            "certificate_test::p",
            "certificate_test::x",
            "certificate_test::y",
            "certificate_test::z",
            "certificate_test::a",
            "certificate_test::b",
            "certificate_test::c",
            "certificate_test::u",
            "certificate_test::v",
            "certificate_test::t",
            "certificate_test::w"
        );
        let ring = Arc::new(Ring::new(vec![p, x, y, z, a, b, c, u, v, t, w], vec![0]).unwrap());
        let witness = [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            .into_iter()
            .map(Rational::from)
            .collect::<Vec<_>>();
        let source = Arc::new(
            Chart::new(ring.clone(), vec![1, 2, 3], vec![], witness.clone(), budget).unwrap(),
        );
        let adapted = Arc::new(
            Chart::new(ring.clone(), vec![4, 5, 6], vec![], witness.clone(), budget).unwrap(),
        );
        let target =
            Arc::new(Chart::new(ring.clone(), vec![7, 8, 9], vec![], witness, budget).unwrap());
        Self {
            ring,
            source,
            adapted,
            target,
        }
    }
    fn p(&self, a: Atom) -> Poly {
        self.ring.atom(&a).unwrap()
    }
    fn frame(&self, budget: &mut Budget) -> FrameCertificate {
        let images = vec![
            self.p(parse!("certificate_test::a")),
            self.p(parse!(
                "certificate_test::b+certificate_test::p*certificate_test::c^2"
            )),
            self.p(parse!("certificate_test::c")),
        ];
        let inverse = vec![
            self.p(parse!("certificate_test::x")),
            self.p(parse!(
                "certificate_test::y-certificate_test::p*certificate_test::z^2"
            )),
            self.p(parse!("certificate_test::z")),
        ];
        let boundaries = vec![
            Boundary {
                id: 7,
                birth_stage: 0,
                multiplicity: 2,
                source_equation: self.p(parse!("certificate_test::x")),
                frame_axis: 4,
                frame_unit: self.ring.one(),
            },
            Boundary {
                id: 8,
                birth_stage: 1,
                multiplicity: 1,
                source_equation: self.p(parse!("certificate_test::z")),
                frame_axis: 6,
                frame_unit: self.ring.one(),
            },
        ];
        FrameCertificate {
            pullback: Map::new(self.source.clone(), self.adapted.clone(), images, budget).unwrap(),
            inverse: Map::new(self.adapted.clone(), self.source.clone(), inverse, budget).unwrap(),
            boundaries,
        }
    }
    fn center(&self, budget: &mut Budget) -> CenterCertificate {
        let ideal=Ideal::new(self.ring.clone(),vec![self.p(parse!("(certificate_test::y-certificate_test::p*certificate_test::z^2)^2+certificate_test::x^3"))],budget).unwrap();
        CenterCertificate {
            frame: self.frame(budget).verify(budget).unwrap(),
            source: MarkedIdeal::new(ideal, 2, budget).unwrap(),
            normal_axes: vec![4, 5],
        }
    }
    fn transform(&self, budget: &mut Budget) -> ControlledTransformCertificate {
        let images = vec![
            self.p(parse!("certificate_test::u")),
            self.p(parse!("certificate_test::u*certificate_test::v")),
            self.p(parse!("certificate_test::t")),
        ];
        let target = MarkedIdeal::new(
            Ideal::new(
                self.ring.clone(),
                vec![self.p(parse!("certificate_test::v^2+certificate_test::u"))],
                budget,
            )
            .unwrap(),
            2,
            budget,
        )
        .unwrap();
        ControlledTransformCertificate {
            center: self.center(budget).verify(budget).unwrap(),
            blowup: Map::new(self.adapted.clone(), self.target.clone(), images, budget).unwrap(),
            pivot_position: 0,
            born_divisor_id: 9,
            born_stage: 2,
            target,
            divisor_transforms: vec![
                DivisorTransform {
                    source_id: 7,
                    birth_stage: 0,
                    source_multiplicity: 2,
                    exceptional_multiplicity: 1,
                    strict_equation: self.ring.one(),
                    pulled_unit: self.ring.one(),
                },
                DivisorTransform {
                    source_id: 8,
                    birth_stage: 1,
                    source_multiplicity: 1,
                    exceptional_multiplicity: 0,
                    strict_equation: self.p(parse!("certificate_test::t")),
                    pulled_unit: self.ring.one(),
                },
            ],
        }
    }
}

#[test]
fn nonlinear_relative_frame_center_coefficient_and_transform_recombine() {
    let mut budget = Budget::new(Limits::default());
    let f = Fixture::new(&mut budget);
    let center = f.center(&mut budget).verify(&mut budget).unwrap();
    assert_eq!(center.frame().jacobian(), &f.ring.one());
    let coefficient = center.coefficient_ideal(5, &mut budget).unwrap();
    assert_eq!(coefficient.marked_ideal.mark(), 2);
    let expected = Ideal::new(
        f.ring.clone(),
        vec![f.p(parse!("certificate_test::a^3"))],
        &mut budget,
    )
    .unwrap();
    for p in coefficient.marked_ideal.ideal().generators() {
        assert!(f.adapted.contains(&expected, p, &mut budget).unwrap());
    }
    for p in expected.generators() {
        assert!(
            f.adapted
                .contains(coefficient.marked_ideal.ideal(), p, &mut budget)
                .unwrap()
        );
    }
    let transform = f.transform(&mut budget).verify(&mut budget).unwrap();
    assert_eq!(
        transform.exceptional_factor(),
        &f.p(parse!("certificate_test::u^2"))
    );
    assert_eq!(transform.jacobian(), &f.p(parse!("certificate_test::u")));
    assert_eq!(
        transform.generator_quotients(),
        &[f.p(parse!("certificate_test::v^2+certificate_test::u"))]
    );
    assert_eq!(transform.born_divisor(), (9, 2));
    assert_eq!(transform.divisor_transforms()[0].source_multiplicity, 2);
}

#[test]
fn wrong_frame_center_boundary_and_contact_witnesses_are_rejected() {
    let mut budget = Budget::new(Limits::default());
    let f = Fixture::new(&mut budget);
    let mut bad = f.frame(&mut budget);
    bad.inverse = Map::new(
        f.adapted.clone(),
        f.source.clone(),
        vec![
            f.p(parse!("certificate_test::x")),
            f.p(parse!("certificate_test::y")),
            f.p(parse!("certificate_test::z")),
        ],
        &mut budget,
    )
    .unwrap();
    assert!(bad.verify(&mut budget).is_err());
    let mut bad = f.frame(&mut budget);
    bad.boundaries[1].frame_axis = 4;
    assert!(bad.verify(&mut budget).is_err());
    let mut bad = f.frame(&mut budget);
    bad.boundaries[0].frame_unit = f.p(parse!("certificate_test::c"));
    assert!(bad.verify(&mut budget).is_err());
    let mut bad = f.center(&mut budget);
    bad.source = MarkedIdeal::new(bad.source.ideal().clone(), 3, &budget).unwrap();
    assert!(bad.verify(&mut budget).is_err());
    let center = f.center(&mut budget).verify(&mut budget).unwrap();
    assert!(center.coefficient_ideal(4, &mut budget).is_err()); // boundary contact
    let mut bad = f.center(&mut budget);
    bad.normal_axes = vec![0];
    assert!(bad.verify(&mut budget).is_err()); // inert parameter cannot be centre axis
}

#[test]
fn transform_rejects_missing_factors_wrong_ideals_divisor_mutation() {
    let mut budget = Budget::new(Limits::default());
    let f = Fixture::new(&mut budget);
    let mut bad = f.transform(&mut budget);
    bad.target = MarkedIdeal::new(
        Ideal::new(
            f.ring.clone(),
            vec![f.p(parse!(
                "certificate_test::u*(certificate_test::v^2+certificate_test::u)"
            ))],
            &mut budget,
        )
        .unwrap(),
        2,
        &budget,
    )
    .unwrap();
    assert!(bad.verify(&mut budget).is_err());
    let mut bad = f.transform(&mut budget);
    bad.divisor_transforms[0].exceptional_multiplicity = 0;
    assert!(bad.verify(&mut budget).is_err());
    let mut bad = f.transform(&mut budget);
    bad.divisor_transforms[0].source_multiplicity = 1;
    assert!(bad.verify(&mut budget).is_err());
    let mut bad = f.transform(&mut budget);
    bad.born_divisor_id = 7;
    assert!(bad.verify(&mut budget).is_err());
    let mut bad = f.transform(&mut budget);
    bad.pivot_position = 2;
    assert!(bad.verify(&mut budget).is_err());
}

#[test]
fn localized_membership_rejects_vacuous_patch_and_preserves_open_scope() {
    let mut budget = Budget::new(Limits::default());
    let f = Fixture::new(&mut budget);
    let mut witness = vec![Rational::from(0); f.ring.len()];
    witness[0] = 2.into();
    witness[2] = 1.into();
    witness[10] = 1.into();
    let chart = Chart::new(
        f.ring.clone(),
        vec![1, 2, 3],
        vec![Guard {
            factor: f.p(parse!("certificate_test::y")),
            inverse_axis: 10,
        }],
        witness.clone(),
        &mut budget,
    )
    .unwrap();
    let ideal = Ideal::new(
        f.ring.clone(),
        vec![f.p(parse!("certificate_test::x*certificate_test::y"))],
        &mut budget,
    )
    .unwrap();
    let x = f.p(parse!("certificate_test::x"));
    assert!(chart.contains(&ideal, &x, &mut budget).unwrap());
    assert!(!f.source.contains(&ideal, &x, &mut budget).unwrap());
    assert!(
        Chart::new(
            f.ring.clone(),
            vec![1, 2, 3],
            vec![Guard {
                factor: x,
                inverse_axis: 10
            }],
            witness,
            &mut budget
        )
        .is_err()
    );
}

#[test]
fn common_marks_and_allocation_limits_are_checked_before_large_constructions() {
    let mut budget = Budget::new(Limits::default());
    let f = Fixture::new(&mut budget);
    let left = MarkedIdeal::new(
        Ideal::new(
            f.ring.clone(),
            vec![
                f.p(parse!("certificate_test::a^2")),
                f.p(parse!("certificate_test::b^2")),
            ],
            &mut budget,
        )
        .unwrap(),
        2,
        &budget,
    )
    .unwrap();
    let right = MarkedIdeal::new(
        Ideal::new(
            f.ring.clone(),
            vec![f.p(parse!("certificate_test::c^3"))],
            &mut budget,
        )
        .unwrap(),
        3,
        &budget,
    )
    .unwrap();
    let sum = left.sum(&right, &mut budget).unwrap();
    assert_eq!(sum.mark(), 6);
    assert!(
        f.adapted
            .contains(
                sum.ideal(),
                &f.p(parse!("certificate_test::a^4*certificate_test::b^2")),
                &mut budget
            )
            .unwrap()
    );
    let mut small = Budget::new(Limits {
        max_mark: 5,
        ..Limits::default()
    });
    assert!(matches!(
        left.sum(&right, &mut small),
        Err(Error::ResourceIncomplete("mark LCM budget"))
    ));
    assert_eq!(small.operations(), 0);
    let mut small = Budget::new(Limits {
        max_generators: 4,
        ..Limits::default()
    });
    assert!(matches!(
        left.ideal().power(3, &mut small),
        Err(Error::ResourceIncomplete(_))
    ));
    assert_eq!(small.operations(), 0);
    let zero = Ideal::new(f.ring.clone(), vec![f.ring.one().zero()], &mut budget).unwrap();
    assert!(zero.generators().is_empty());
    assert!(
        !f.adapted
            .contains(&zero, &f.ring.one(), &mut budget)
            .unwrap()
    );
    let mut small = Budget::new(Limits {
        max_total_ideal_slots: 2,
        ..Limits::default()
    });
    assert!(matches!(
        left.ideal().product(left.ideal(), &mut small),
        Err(Error::ResourceIncomplete("total ideal allocation budget"))
    ));
    assert_eq!(small.ideal_slots(), 0);
}

#[test]
fn companion_extracts_recorded_boundary_power_without_changing_history() {
    let mut budget = Budget::new(Limits::default());
    let f = Fixture::new(&mut budget);
    let mut certificate = f.center(&mut budget);
    certificate.source=MarkedIdeal::new(Ideal::new(f.ring.clone(),vec![f.p(parse!("certificate_test::x^2*((certificate_test::y-certificate_test::p*certificate_test::z^2)^2+certificate_test::x^3)"))],&mut budget).unwrap(),3,&budget).unwrap();
    let center = certificate.verify(&mut budget).unwrap();
    let result = center.companion(vec![(7, 2)], 2, &mut budget).unwrap();
    assert_eq!(result.monomial_factor, f.p(parse!("certificate_test::a^2")));
    assert_eq!(result.verified_local_residual_order, 2);
    let companion = result.companion.unwrap();
    assert_eq!(companion.mark(), 2);
    assert!(
        f.adapted
            .contains(
                companion.ideal(),
                &f.p(parse!("certificate_test::a^4")),
                &mut budget
            )
            .unwrap()
    );
    assert_eq!(center.frame().boundaries()[0].multiplicity, 2);
    assert!(center.companion(vec![(7, 1)], 2, &mut budget).is_err());
    assert!(center.companion(vec![(7, 2)], 3, &mut budget).is_err());
}
