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
    match load(&bytes) {
        Ok(_) => panic!("invalid native payload admitted"),
        Err(e) => e.to_string(),
    }
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
