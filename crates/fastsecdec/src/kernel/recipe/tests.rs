use super::check_source::DynamicCheckOutput;
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
fn staging_descriptor_rejects_truncation_trailing_data_and_wrong_version() {
    let owner = dynamic_owner(3);
    let bytes = owner.to_staging_bytes().unwrap();
    drop(owner);
    let restored = NativeProgramDescriptor::from_staging_bytes(&bytes).unwrap();
    assert_eq!(restored.charts()[0].chart_index, 3);
    assert!(NativeProgramDescriptor::from_staging_bytes(&bytes[..bytes.len() - 1]).is_err());
    let mut trailing = bytes;
    trailing.push(0);
    assert!(NativeProgramDescriptor::from_staging_bytes(&trailing).is_err());
    let mut wrong = restored.to_staging_bytes().unwrap();
    wrong[0] = 99; // First field is the small unsigned schema version.
    assert!(NativeProgramDescriptor::from_staging_bytes(&wrong).is_err());
}

fn contour_metadata() -> crate::generation::GenerationMetadata {
    use crate::{
        generation::{GenerationOptions, generate},
        parametric::*,
    };
    let x = symbol!("recipe_payload::x");
    let source = ParametricIntegrand::new(
        vec![x],
        symbol!("recipe_payload::eps"),
        ParametricDomain::UnitCube,
        vec![ParametricTerm::new(
            Atom::one(),
            vec![Atom::Zero],
            vec![
                PolynomialFactor::new(
                    Atom::num(2) + Atom::var(x).pow(5),
                    Atom::num(-1),
                    FactorRole::Singularity,
                )
                .with_semantics(FactorSemantics::Causal),
            ],
        )],
    )
    .unwrap();
    generate(
        &source,
        &GenerationOptions {
            program_recipe: ProgramRecipe::FixedV1,
            ..Default::default()
        },
        |_| std::ops::ControlFlow::Continue(()),
    )
    .unwrap()
    .metadata()
    .clone()
}

#[test]
fn descriptor_binds_chart_dimension_and_positive_factor_count_before_runtime_admission() {
    let metadata = contour_metadata();
    let chart = &metadata.charts()[0];
    let contour = chart.contour().unwrap();
    let envelope = DynamicEnvelope::new(
        chart.coordinates().target_parameters(),
        contour.causal_polynomial().clone(),
        contour.positive_polynomials(),
    )
    .unwrap();
    let helper = RootProgram::build(envelope.maximum_even_order() as usize / 2).unwrap();
    let record =
        DynamicChartRecipe::from_envelope(chart.source_index(), &envelope, &helper).unwrap();
    let owner = NativeProgramDescriptor::dynamic(
        ProgramRecipe::DynamicPolynomialV1,
        vec![record],
        vec![helper],
    )
    .unwrap();
    let runtime = ProgramRecipe::DynamicPolynomialV1
        .recipe_parameters()
        .iter()
        .map(|name| symbol!(*name))
        .collect::<Vec<_>>();
    owner
        .validate_generation(Some(&metadata), &runtime)
        .unwrap();
    let mut wrong = owner.clone();
    wrong.charts[0].dimension += 1;
    assert!(
        wrong
            .validate_generation(Some(&metadata), &runtime)
            .is_err()
    );
    let mut wrong = owner.clone();
    wrong.charts[0].chart_index += 1;
    assert!(
        wrong
            .validate_generation(Some(&metadata), &runtime)
            .is_err()
    );
    assert!(owner.validate_generation(None, &runtime).is_err());
    assert!(owner.validate_generation(Some(&metadata), &[]).is_err());
    let mut wrong = metadata.clone();
    wrong.charts[0]
        .contour
        .as_mut()
        .unwrap()
        .positive_polynomials
        .push(Atom::one());
    assert!(owner.validate_generation(Some(&wrong), &runtime).is_err());

    let mut symmetric = metadata;
    let mut copy = symmetric.charts[0].clone();
    copy.source_index = 1;
    symmetric.charts.push(copy);
    owner
        .validate_generation(Some(&symmetric), &runtime)
        .unwrap();
    let selected = owner.for_payload(&[0, 1], &[], Some(&symmetric)).unwrap();
    assert_eq!(selected.charts().len(), 1);
    assert!(owner.for_payload(&[1], &[], Some(&symmetric)).is_err());
    assert!(owner.for_payload(&[0, 2], &[], Some(&symmetric)).is_err());
}

#[test]
fn check_sources_keep_full_structure_and_an_independent_strength_input() {
    let x = symbol!("recipe_check_source::x");
    let envelope = DynamicEnvelope::new(
        &[x],
        Atom::num(2) + Atom::var(x).pow(5),
        &[Atom::one() + Atom::var(x).pow(2)],
    )
    .unwrap();
    let source = DynamicCheckSource::from_envelope(4, &envelope);
    let helper = RootProgram::build(envelope.maximum_even_order() as usize / 2).unwrap();
    let chart = DynamicChartRecipe::from_envelope(4, &envelope, &helper).unwrap();
    let descriptor = NativeProgramDescriptor::dynamic(
        ProgramRecipe::DynamicPolynomialV1,
        vec![chart],
        vec![helper],
    )
    .unwrap();
    descriptor
        .validate_sources(&[std::sync::Arc::new(source.clone())], None)
        .unwrap();
    assert!(descriptor.validate_sources(&[], None).is_err());
    let mut truncated = source.clone();
    truncated.outputs.pop();
    assert!(
        descriptor
            .validate_sources(&[std::sync::Arc::new(truncated)], None)
            .is_err()
    );
    let mut reordered = source.clone();
    reordered.schema.swap(0, 1);
    assert!(
        descriptor
            .validate_sources(&[std::sync::Arc::new(reordered)], None)
            .is_err()
    );
    assert_eq!(source.outputs.len(), source.schema.len());
    assert_eq!(source.chart_index, 4);
    let index = source
        .schema
        .iter()
        .position(|kind| matches!(kind, DynamicCheckOutput::CausalRay))
        .unwrap();
    let ray = &source.outputs[index];
    assert!(ray.contains_symbol(crate::contour::lambda_symbol()));
    assert!(!ray.to_canonical_string().contains("strength_v1"));
    let lambda = Atom::var(crate::contour::lambda_symbol());
    let at_origin = ray.replace(lambda).with(Atom::Zero);
    assert_eq!(at_origin, *envelope.causal_polynomial());
    assert!(
        source
            .schema
            .contains(&DynamicCheckOutput::SpectralGapSquared { order: 5 })
    );
    assert!(
        source
            .schema
            .contains(&DynamicCheckOutput::PositiveHarmfulCoefficient {
                factor: 0,
                order: 2
            })
    );
}

#[test]
fn selected_charts_release_unneeded_helpers_and_preserve_explicit_recipe() {
    let owner = dynamic_owner(7);
    let mut selected = owner.for_payload(&[7], &[], None).unwrap();
    assert_eq!(selected.charts()[0].chart_index, 0);
    assert_eq!(selected.helpers.len(), 1);
    selected.remap_charts(&[7]).unwrap();
    assert_eq!(selected.charts()[0].chart_index, 7);
    assert!(selected.clone().remap_charts(&[]).is_err());
    let empty = owner.for_payload(&[], &[], None).unwrap();
    assert!(empty.charts().is_empty());
    assert!(empty.helpers.is_empty());
    assert_eq!(empty.recipe(), ProgramRecipe::DynamicPolynomialV1);
    assert!(owner.for_payload(&[8], &[], None).is_err());
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
    let exact_owner = owner.for_payload(&[], &[exact], None).unwrap();
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
fn exact_aggregation_merges_only_referenced_native_helpers_without_chart_residency() {
    let owner = dynamic_owner(7);
    let helper = &owner.helpers[0];
    let exact = crate::contour::functions::dynamic::strength(
        helper,
        &vec![Atom::one(); helper.coefficient_count()],
        &Atom::num((1, 2)),
        &Atom::one(),
    )
    .unwrap();
    let mut aggregate = owner.for_payload(&[], &[exact], None).unwrap();

    let envelope = DynamicEnvelope::new(&[], Atom::one(), &[]).unwrap();
    let helper = RootProgram::build(1).unwrap();
    let exact = crate::contour::functions::dynamic::strength(
        &helper,
        &[Atom::num(4)],
        &Atom::num((1, 2)),
        &Atom::one(),
    )
    .unwrap();
    let chart = DynamicChartRecipe::from_envelope(9, &envelope, &helper).unwrap();
    let other = NativeProgramDescriptor::dynamic(
        ProgramRecipe::DynamicPolynomialV1,
        vec![chart],
        vec![helper],
    )
    .unwrap();
    let selected = other.for_payload(&[], &[exact], None).unwrap();
    aggregate.merge(selected.clone()).unwrap();
    aggregate.merge(selected).unwrap();
    assert!(aggregate.charts().is_empty());
    assert_eq!(aggregate.helpers.len(), 2);
    assert_eq!(aggregate.exact_helpers.len(), 2);
    let restored =
        NativeProgramDescriptor::from_staging_bytes(&aggregate.to_staging_bytes().unwrap())
            .unwrap();
    assert!(restored.charts().is_empty());
    assert_eq!(restored.helpers.len(), 2);
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
