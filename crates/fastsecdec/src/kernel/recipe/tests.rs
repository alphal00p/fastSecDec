use super::*;
use crate::contour::dynamic::DynamicEnvelope;
use symbolica::{atom::Atom, symbol};

fn dynamic_owner(chart_index: usize) -> NativeProgramDescriptor {
    let x = symbol!("fastsecdec::tests::recipe_x");
    let envelope = DynamicEnvelope::new(
        &[x],
        Atom::num(2) + Atom::var(x).pow(5),
        &[Atom::num(1) + Atom::var(x)],
    )
    .unwrap();
    let helper = RootProgram::build(envelope.maximum_even_order() as usize / 2).unwrap();
    let chart = DynamicChartRecipe::from_envelope(chart_index, &envelope, &helper).unwrap();
    NativeProgramDescriptor::dynamic(
        ProgramRecipe::DynamicPolynomialV1,
        vec![chart],
        vec![helper],
    )
    .unwrap()
}

#[test]
fn saved_descriptor_restores_native_helpers_and_full_sector_counts() {
    let owner = dynamic_owner(7);
    let saved = SavedProgramDescriptor::from_native(&owner);
    let bytes = bincode::serde::encode_to_vec(&saved, bincode::config::standard()).unwrap();
    drop(owner);
    let (saved, used): (SavedProgramDescriptor, _) =
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard()).unwrap();
    assert_eq!(used, bytes.len());
    let restored = saved.restore().unwrap();
    assert_eq!(restored.recipe(), ProgramRecipe::DynamicPolynomialV1);
    assert_eq!(restored.charts()[0].chart_index, 7);
    assert_eq!(restored.charts()[0].causal_orders, vec![3, 5]);
    assert_eq!(
        restored.helpers[0].coefficient_count() * 2,
        restored.charts()[0].maximum_even_order as usize
    );
    assert!(
        restored
            .admit_runtime()
            .unwrap_err()
            .to_string()
            .contains("production admission")
    );
}

#[test]
fn selected_charts_release_unneeded_helpers_and_preserve_explicit_recipe() {
    let owner = dynamic_owner(7);
    let selected = owner.for_payload(&[7], &[]).unwrap();
    assert_eq!(selected.charts()[0].chart_index, 0);
    assert_eq!(selected.helpers.len(), 1);
    let empty = owner.for_payload(&[], &[]).unwrap();
    assert!(empty.charts().is_empty());
    assert!(empty.helpers.is_empty());
    assert_eq!(empty.recipe(), ProgramRecipe::DynamicPolynomialV1);
    assert!(owner.for_payload(&[8], &[]).is_err());
}

#[test]
fn compact_exact_offsets_retain_only_their_referenced_helper_owners() {
    let owner = dynamic_owner(7);
    let helper = &owner.helpers[0];
    let coefficients = vec![Atom::one(); helper.coefficient_count()];
    let exact = crate::contour::functions::dynamic::strength(
        helper,
        &coefficients,
        &Atom::num((1, 2)),
        &Atom::one(),
    )
    .unwrap();
    let exact_owner = owner.for_payload(&[], &[exact]).unwrap();
    assert!(exact_owner.charts().is_empty());
    assert_eq!(exact_owner.helpers.len(), 1);
    assert_eq!(exact_owner.exact_helpers, vec![helper.digest().to_owned()]);
    let restored = SavedProgramDescriptor::from_native(&exact_owner)
        .restore()
        .unwrap();
    assert_eq!(restored.helpers.len(), 1);
    assert!(restored.charts().is_empty());
}

#[test]
fn descriptor_rejects_wrong_schema_or_unreferenced_owner() {
    let mut owner = dynamic_owner(0);
    owner.charts[0].maximum_even_order += 2;
    assert!(owner.validate().is_err());
    let mut owner = dynamic_owner(0);
    owner.charts[0].positive_proofs[0] = PositiveFactorProof::NonnegativeCoefficients {
        lower_bound: Rational::from(0),
    };
    assert!(owner.validate().is_err());
    let mut owner = dynamic_owner(0);
    owner.helpers.push(RootProgram::build(1).unwrap());
    assert!(owner.validate().is_err());
    assert!(NativeProgramDescriptor::static_recipe(ProgramRecipe::DynamicSignAwareV1).is_err());
}

#[test]
fn zero_dimensional_exact_chart_keeps_the_dynamic_recipe() {
    let envelope = DynamicEnvelope::new(&[], Atom::num(-1), &[Atom::num(2)]).unwrap();
    let helper = RootProgram::build(1).unwrap();
    let chart = DynamicChartRecipe::from_envelope(0, &envelope, &helper).unwrap();
    assert_eq!(chart.dimension, 0);
    assert!(chart.causal_orders.is_empty());
    assert_eq!(chart.maximum_even_order, 2);
    let owner = NativeProgramDescriptor::dynamic(
        ProgramRecipe::DynamicPolynomialV1,
        vec![chart],
        vec![helper],
    )
    .unwrap();
    let restored = SavedProgramDescriptor::from_native(&owner)
        .restore()
        .unwrap();
    assert_eq!(restored.recipe(), ProgramRecipe::DynamicPolynomialV1);
    assert_eq!(restored.charts()[0].dimension, 0);
}
