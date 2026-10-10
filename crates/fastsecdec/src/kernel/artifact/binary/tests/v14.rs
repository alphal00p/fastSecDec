use super::*;
use crate::parametric::{FactorRole, PolynomialFactor};

#[test]
fn subset_scope_shape_is_mandatory_and_semantic_identity_validation_is_explicit() {
    let input = ParametricIntegrand::new(
        vec![symbol!("subset_wire::x"), symbol!("subset_wire::y")],
        symbol!("subset_wire::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero; 2],
            vec![PolynomialFactor::new(
                parse!("subset_wire::x+subset_wire::y"),
                Atom::num(-1),
                FactorRole::Singularity,
            )],
        )],
    )
    .unwrap();
    let generated = generate(
        &input,
        &GenerationOptions {
            source_sectors: Some(vec![1]),
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let original = generated
        .to_kernel_bytes(PrecisionPolicy::default())
        .unwrap();
    for malformed_shape in [false, true] {
        let (mut envelope, _): (Envelope, usize) = bincode::decode_from_slice(
            original.strip_prefix(MAGIC_V14).unwrap(),
            bincode::config::standard(),
        )
        .unwrap();
        let state = State::import(&mut envelope.state.as_slice(), None).unwrap();
        let (mut payload, _): (PayloadV14, usize) = bincode::decode_from_slice_with_context(
            &envelope.payload,
            bincode::config::standard(),
            state,
        )
        .unwrap();
        payload.scope = serde_json::from_value(serde_json::json!({
            "selection":{"original_source_count":2,"source_sectors":[0]},
            "chart_source_sectors": if malformed_shape { vec![] } else { vec![0] }
        }))
        .unwrap();
        envelope.payload = bincode::encode_to_vec(payload, bincode::config::standard()).unwrap();
        envelope.digest = *digest(MAGIC_V14, &envelope.state, &envelope.payload).as_bytes();
        let mut bytes = MAGIC_V14.to_vec();
        bytes.extend(bincode::encode_to_vec(envelope, bincode::config::standard()).unwrap());
        assert!(
            KernelSet::from_bytes_with_options(&bytes, KernelLoadOptions { validate: true })
                .is_err()
        );
        assert_eq!(
            KernelSet::from_bytes_with_options(&bytes, KernelLoadOptions { validate: false })
                .is_err(),
            malformed_shape
        );
    }
}
