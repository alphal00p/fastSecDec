use super::*;
use crate::contour::{FixedContourMap, lambda_symbol};
use symbolica::parse;

#[test]
fn templates_remove_false_pivots_and_preserve_exact_singular_matrices() {
    for dimension in FIRST_TEMPLATE..=LAST_TEMPLATE {
        let mut entries = vec![Atom::Zero; dimension * dimension];
        entries[1] = Atom::one();
        entries[dimension] = Atom::one();
        for i in 2..dimension {
            entries[i * dimension + i] = Atom::one();
        }
        // First pivot zero, matrix nonsingular: no 0/0 survives substitution.
        assert_eq!(
            determinant(entries.clone(), dimension as u32).unwrap(),
            Atom::num(-1)
        );
        assert_eq!(native(entries, dimension as u32).unwrap(), Atom::num(-1));
        // A vanishing Jacobian is a legitimate algebraic result, not an error.
        assert!(
            determinant(vec![Atom::one(); dimension * dimension], dimension as u32)
                .unwrap()
                .is_zero()
        );
        let template = TEMPLATES[dimension - FIRST_TEMPLATE]
            .get()
            .unwrap()
            .as_ref()
            .unwrap();
        assert!(template.determinant.is_polynomial(true, false).is_some());
    }
}

#[test]
fn physical_entries_are_substituted_before_symbolic_derivatives() {
    let x = symbol!("contour_determinant_test::x");
    let variable = Atom::var(x);
    for dimension in FIRST_TEMPLATE..=LAST_TEMPLATE {
        let mut entries = vec![Atom::Zero; dimension * dimension];
        for i in 0..dimension {
            entries[i * dimension + i] = Atom::one();
        }
        entries[0] = variable.pow(2);
        entries[1] = Atom::one();
        entries[dimension] = variable.clone();
        let determinant = determinant(entries, dimension as u32).unwrap();
        assert_eq!(determinant, variable.pow(2) - &variable);
        assert_eq!(determinant.derivative(x), Atom::num(2) * &variable - 1);
        assert_eq!(determinant.derivative(x).derivative(x), Atom::num(2));
        assert_eq!(
            determinant.series(x, 0, 2).unwrap().to_atom(),
            variable.pow(2) - &variable
        );
    }
}

#[test]
fn six_dimensional_gradient_map_matches_native_exact_matrix_controls() {
    let parameters = (0..6)
        .map(|i| symbol!(format!("contour_determinant_test::x{i}")))
        .collect::<Vec<_>>();
    let f = parse!(
        "1-7*contour_determinant_test::x0*contour_determinant_test::x1
        +3/7*contour_determinant_test::x1*contour_determinant_test::x2
        +5/11*contour_determinant_test::x2*contour_determinant_test::x3*contour_determinant_test::x4
        -2/13*contour_determinant_test::x4*contour_determinant_test::x5
        +(contour_determinant_test::x0+contour_determinant_test::x3+contour_determinant_test::x5)^3"
    );
    let map = FixedContourMap::new(&parameters, f).unwrap();
    let entries = map
        .metadata()
        .images()
        .iter()
        .flat_map(|image| parameters.iter().map(|p| image.derivative(*p)))
        .collect::<Vec<_>>();
    for origin in [0_i64, 1, 2] {
        let substitute = |expression: &Atom| {
            expression
                .replace_multiple(parameters.iter().enumerate().map(|(i, parameter)| {
                    Replacement::new(
                        Pattern::Literal(Atom::var(*parameter)),
                        Pattern::Literal(Atom::num((origin + i as i64, 11))),
                    )
                }))
                .replace(lambda_symbol())
                .with(Atom::num((1, 17)))
        };
        let reference = native(entries.iter().map(substitute).collect(), 6).unwrap();
        assert!(
            (substitute(map.metadata().jacobian()) - reference)
                .expand()
                .cancel()
                .is_zero()
        );
    }
    assert_eq!(
        map.metadata().jacobian().replace(lambda_symbol()).with(0),
        Atom::one()
    );
}

#[test]
fn determinant_preserves_small_and_large_native_paths_and_checks_shape() {
    for dimension in [0, 1, 2, 3, 7] {
        let entries = (0..dimension * dimension)
            .map(|i| {
                if i / dimension == i % dimension {
                    Atom::one()
                } else {
                    Atom::Zero
                }
            })
            .collect();
        assert_eq!(determinant(entries, dimension).unwrap(), Atom::one());
    }
    assert!(determinant(vec![Atom::one(); 15], 4).is_err());
}
