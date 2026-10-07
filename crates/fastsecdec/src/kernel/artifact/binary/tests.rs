//! Malformed semantic records use the actual private binserde owner and encoder.
//! Public integration tests separately cover corruption, exhaustion and science.
use super::*;
use crate::{
    generation::{GenerationOptions, generate},
    parametric::{ParametricDomain, ParametricIntegrand, ParametricTerm},
};
use std::ops::ControlFlow;
use symbolica::{parse, symbol};

fn kernel(prefactor: Atom) -> KernelSet {
    let input = ParametricIntegrand::new(
        vec![symbol!("binary_checks::x")],
        symbol!("binary_checks::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(prefactor, vec![Atom::one()], vec![])],
    )
    .unwrap();
    generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap()
    .compile()
    .unwrap()
}
fn payload(kernels: &KernelSet) -> Payload {
    let bytes = kernels.artifact_bytes().unwrap();
    let (envelope, used): (Envelope, usize) = bincode::decode_from_slice(
        bytes.strip_prefix(MAGIC).unwrap(),
        bincode::config::standard(),
    )
    .unwrap();
    assert_eq!(used, bytes.len() - MAGIC.len());
    let map = State::import(&mut envelope.state.as_slice(), None).unwrap();
    let (payload, used) = bincode::decode_from_slice_with_context(
        &envelope.payload,
        bincode::config::standard(),
        map,
    )
    .unwrap();
    assert_eq!(used, envelope.payload.len());
    payload
}
fn rejected(payload: Payload) -> String {
    let (_, bytes) = encode(payload).unwrap();
    let mut error = String::new();
    for validate in [false, true] {
        error = match KernelSet::from_bytes_with_options(&bytes, KernelLoadOptions { validate }) {
            Ok(_) => panic!("invalid native payload admitted (validate={validate})"),
            Err(e) => e.to_string(),
        };
    }
    error
}

#[test]
fn borrowed_envelope_preserves_wire_and_borrows_large_buffers() {
    let kernels = super::super::load_tests::template();
    let bytes = kernels.artifact_bytes().unwrap();
    let wire = bytes.strip_prefix(MAGIC).unwrap();
    let (borrowed, used): (EnvelopeRef<'_>, _) =
        bincode::borrow_decode_from_slice(wire, bincode::config::standard()).unwrap();
    assert_eq!(used, wire.len());
    let (owned, used): (Envelope, _) =
        bincode::decode_from_slice(wire, bincode::config::standard()).unwrap();
    assert_eq!(used, wire.len());
    assert_eq!(borrowed.content_id, owned.content_id);
    assert_eq!(borrowed.state, owned.state);
    assert_eq!(borrowed.payload, owned.payload);
    for buffer in [borrowed.state, borrowed.payload] {
        assert!(buffer.as_ptr() >= wire.as_ptr());
        assert!(buffer.as_ptr_range().end <= wire.as_ptr_range().end);
    }
}

#[test]
fn binary_content_checks_are_explicit_and_format_checks_are_unconditional() {
    let kernels = super::super::load_tests::template();
    let original = kernels.artifact_bytes().unwrap();
    for change_digest in [false, true] {
        let (mut envelope, _): (Envelope, _) = bincode::decode_from_slice(
            original.strip_prefix(MAGIC).unwrap(),
            bincode::config::standard(),
        )
        .unwrap();
        if change_digest {
            envelope.digest[0] ^= 1;
        } else {
            envelope.content_id = "f".repeat(64);
        }
        let mut bytes = MAGIC.to_vec();
        bytes.extend(bincode::encode_to_vec(envelope, bincode::config::standard()).unwrap());
        assert!(KernelSet::from_bytes(&bytes).is_ok());
        assert!(
            KernelSet::from_bytes_with_options(&bytes, KernelLoadOptions { validate: true })
                .is_err()
        );
    }
    let (mut envelope, _): (Envelope, _) = bincode::decode_from_slice(
        original.strip_prefix(MAGIC).unwrap(),
        bincode::config::standard(),
    )
    .unwrap();
    envelope.content_id = "malformed".into();
    let mut malformed_id = MAGIC.to_vec();
    malformed_id.extend(bincode::encode_to_vec(envelope, bincode::config::standard()).unwrap());
    let mut trailing = original.to_vec();
    trailing.push(0);
    for validate in [false, true] {
        for bytes in [
            malformed_id.as_slice(),
            trailing.as_slice(),
            &original[..original.len() - 1],
        ] {
            assert!(
                KernelSet::from_bytes_with_options(bytes, KernelLoadOptions { validate }).is_err()
            );
        }
    }
}

fn historical_v7(payload: Payload) -> Vec<u8> {
    let content_id = semantic_id(&payload, 7).unwrap();
    let (_, bytes) = encode(payload).unwrap();
    let (mut envelope, _): (Envelope, usize) = bincode::decode_from_slice(
        bytes.strip_prefix(MAGIC).unwrap(),
        bincode::config::standard(),
    )
    .unwrap();
    envelope.content_id = content_id;
    envelope.digest = *digest(MAGIC_V7, &envelope.state, &envelope.payload).as_bytes();
    let mut bytes = MAGIC_V7.to_vec();
    bytes.extend(bincode::encode_to_vec(envelope, bincode::config::standard()).unwrap());
    bytes
}

#[test]
fn runtime_branch_historical_real_artifacts_require_regeneration() {
    let x = symbol!("binary_branch::x");
    let p = symbol!("binary_branch::p");
    for coefficient in [
        parse!("binary_branch::p^(1/2)"),
        parse!("log(binary_branch::p)"),
    ] {
        let input = ParametricIntegrand::new(
            vec![x],
            symbol!("binary_branch::eps"),
            ParametricDomain::UnitCube,
            vec![ParametricTerm::new(coefficient, vec![Atom::one()], vec![])],
        )
        .unwrap();
        let template = generate(&input, &GenerationOptions::default(), |_| {
            ControlFlow::Continue(())
        })
        .unwrap()
        .compile_with_settings_parameters_and_progress(
            Default::default(),
            &[p],
            CompilationSettings {
                backend: crate::kernel::EvaluatorBackend::Eager,
                ..Default::default()
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        let mut record = payload(&template);
        assert_eq!(record.components, native::component_layout(1, true));
        record.components = native::component_layout(1, false);
        let error = match load(&historical_v7(record)) {
            Ok(_) => panic!("historical real branch template admitted"),
            Err(error) => error.to_string(),
        };
        assert!(
            error.contains("regenerate with the current compiler"),
            "{error}"
        );

        let mut old_complex = load(&historical_v7(payload(&template))).unwrap();
        old_complex
            .bind_parameters(&std::collections::BTreeMap::from([(p, -4.0)]))
            .unwrap();
        let mut output = [0.0; 2];
        old_complex.sectors_mut()[0]
            .evaluate(&[0.25], &mut output)
            .unwrap();
        assert!(output.iter().all(|value| value.is_finite()) && output[1] != 0.0);
    }
    let real = kernel(Atom::num(2));
    let mut old_real = load(&historical_v7(payload(&real))).unwrap();
    let mut output = [0.0];
    old_real.sectors_mut()[0]
        .evaluate(&[0.25], &mut output)
        .unwrap();
    assert_eq!(output, [0.5]);
}

#[test]
fn runtime_branch_exact_offset_cannot_be_restored_as_real() {
    let eps = symbol!("binary_branch_exact::eps");
    let p = symbol!("binary_branch_exact::p");
    let input = ParametricIntegrand::new(
        vec![symbol!("binary_branch_exact::x")],
        eps,
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            parse!("binary_branch_exact::p^(1/2)"),
            vec![Atom::var(eps) - 1],
            vec![],
        )],
    )
    .unwrap();
    let template = generate(&input, &GenerationOptions::default(), |_| {
        ControlFlow::Continue(())
    })
    .unwrap()
    .compile_with_settings_parameters_and_progress(
        Default::default(),
        &[p],
        CompilationSettings {
            backend: crate::kernel::EvaluatorBackend::Eager,
            ..Default::default()
        },
        |_| ControlFlow::Continue(()),
    )
    .unwrap();
    let mut record = payload(&template);
    record.components = native::component_layout(record.orders.len(), false);
    assert!(rejected(record).contains("complex exact offset in real output layout"));
}
#[test]
fn exact_serialized_program_size_is_the_native_program_bytes() {
    let kernels = kernel(parse!("2+3𝑖"));
    let payload = payload(&kernels);
    for (sector, stored) in kernels.sectors().iter().zip(payload.sectors) {
        assert_eq!(
            sector.statistics().exact_program_bytes,
            stored.program.len()
        );
    }
}
#[test]
fn wrong_codec_or_policy_precedes_native_program_decode() {
    let kernels = kernel(Atom::one());
    for codec in [true, false] {
        let mut record = payload(&kernels);
        if codec {
            record.codec = "foreign numerical codec".into();
        } else {
            record.compiler_policy = "foreign execution policy".into();
        }
        record.sectors[0].program = vec![255];
        assert!(rejected(record).contains("codec or compiler policy"));
    }
}
#[test]
fn native_program_decoder_requires_the_exact_byte_extent() {
    let kernels = kernel(Atom::one());
    let original = payload(&kernels).sectors.remove(0).program;
    let mut trailing = original.clone();
    trailing.push(0);
    for bad in [
        Vec::new(),
        original[..original.len() - 1].to_vec(),
        trailing,
    ] {
        let mut record = payload(&kernels);
        record.sectors[0].program = bad;
        assert!(!rejected(record).is_empty());
    }
}
#[test]
fn exact_complex_layout_cannot_be_projected_to_real_below_f64_range() {
    let kernels = kernel(parse!("1+𝑖/10^400"));
    let mut record = payload(&kernels);
    assert_eq!(record.components, native::component_layout(1, true));
    record.components = native::component_layout(1, false);
    assert!(rejected(record).contains("complex native coefficient"));
}
#[cfg(feature = "native")]
#[test]
fn fixed_complex_gamma_cannot_be_reinterpreted_as_real() {
    let mut kernels = kernel(parse!("gamma(1+𝑖)"));
    let mut output = [0.0; 2];
    kernels.sectors_mut()[0]
        .evaluate(&[0.25], &mut output)
        .unwrap();
    assert!(output.iter().all(|v| v.is_finite()) && output[1] != 0.0);
    let mut record = payload(&kernels);
    record.components = native::component_layout(1, false);
    assert!(!rejected(record).is_empty());
}
