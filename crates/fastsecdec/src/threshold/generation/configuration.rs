//! Native configuration context. Reuse the ordinary generation-options DTO and
//! State/Atom codec; no expression strings or second source serializer.
use super::*;
use crate::generation::streaming::{
    RecordRef, StoredGenerationOptions,
    codec::{self, Atoms},
};
use symbolica::atom::Atom;
const KIND: &str = "threshold-preparation-configuration-v1";
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Stored {
    generation: StoredGenerationOptions,
    precision: PrecisionPolicy,
    compilation: CompilationSettings,
    bindings: usize,
    max_charts: usize,
    max_degree: u32,
    source_nodes: usize,
    source_depth: usize,
    degree: u16,
    terms: usize,
    integer_power: i64,
    witness_candidates: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    represented: Option<RepresentedConfiguration>,
}
/// Optional, explicitly versioned extension. Exact v1 records omit this field;
/// older strict readers refuse the new represented capability rather than
/// silently restoring it as exact-only.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RepresentedConfiguration {
    version: u32,
    meaning: represented::NumericalMeaning,
    limits: represented::Limits,
}
pub(super) fn write(root: &Path, options: &PreparationOptions) -> Result<RecordRef> {
    let mut symbols = vec![options.unit];
    let mut atoms = Atoms::default();
    for (s, q) in &options.fixed_fiber {
        symbols.push(*s);
        atoms.push(&Atom::num(q.clone()));
    }
    let l = options.regularization;
    let p = l.prefactors;
    Ok(codec::write(
        root,
        "threshold-configuration",
        KIND,
        &Stored {
            generation: (&options.generation).into(),
            precision: options.precision.clone(),
            compilation: options.compilation,
            bindings: options.fixed_fiber.len(),
            max_charts: l.max_charts,
            max_degree: l.max_degree,
            source_nodes: p.source_nodes,
            source_depth: p.source_depth,
            degree: p.degree,
            terms: p.terms,
            integer_power: p.integer_power,
            witness_candidates: p.witness_candidates,
            represented: options
                .represented
                .map(|(meaning, limits)| RepresentedConfiguration {
                    version: 1,
                    meaning,
                    limits,
                }),
        },
        atoms,
        symbols,
    )?)
}
pub(super) fn read(root: &Path, reference: &RecordRef, maximum: u64) -> Result<PreparationOptions> {
    if reference.bytes > maximum {
        return Err(Error::Association("configuration transport limit"));
    }
    let (stored, atoms, symbols): (Stored, _, _) = codec::read(root, reference, KIND)?;
    let representation = match stored.represented {
        Some(value)
            if value.version == 1
                && value.meaning == represented::NumericalMeaning::RepresentedValues =>
        {
            Some((value.meaning, value.limits))
        }
        Some(_) => {
            return Err(Error::Association(
                "unsupported represented configuration version or meaning",
            ));
        }
        None => None,
    };
    if symbols.len()
        != stored
            .bindings
            .checked_add(1)
            .ok_or(Error::Association("configuration binding count overflow"))?
        || atoms.atoms.len() != stored.bindings
    {
        return Err(Error::Association("configuration native table layout"));
    }
    let mut fixed_fiber = BTreeMap::new();
    for (s, a) in symbols[1..].iter().zip(atoms.atoms) {
        let q = Rational::try_from(a.as_view())
            .map_err(|_| Error::Association("nonrational stored fixed fiber"))?;
        if *s == symbols[0] || fixed_fiber.insert(*s, q).is_some() {
            return Err(Error::Association("duplicate fixed fiber/unit role"));
        }
    }
    Ok(PreparationOptions {
        threshold: Default::default(),
        generation: stored.generation.into(),
        precision: stored.precision,
        compilation: stored.compilation,
        unit: symbols[0],
        fixed_fiber,
        represented: representation,
        regularization: regularization::Limits {
            max_charts: stored.max_charts,
            max_degree: stored.max_degree,
            prefactors: regularization::meromorphic::Limits {
                source_nodes: stored.source_nodes,
                source_depth: stored.source_depth,
                degree: stored.degree,
                terms: stored.terms,
                integer_power: stored.integer_power,
                witness_candidates: stored.witness_candidates,
            },
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    // Frozen pre-extension metadata. The exact path must still encode these
    // same fields in this order, without a version/policy upgrade.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct LegacyStored {
        generation: StoredGenerationOptions,
        precision: PrecisionPolicy,
        compilation: CompilationSettings,
        bindings: usize,
        max_charts: usize,
        max_degree: u32,
        source_nodes: usize,
        source_depth: usize,
        degree: u16,
        terms: usize,
        integer_power: i64,
        witness_candidates: usize,
    }
    #[test]
    fn exact_configuration_keeps_legacy_metadata_and_native_record_bytes() {
        let root = tempfile::tempdir().unwrap();
        let options = PreparationOptions::new(symbolica::symbol!("legacy_configuration::t"));
        let reference = write(root.path(), &options).unwrap();
        let (stored, atoms, symbols): (Stored, _, _) =
            codec::read(root.path(), &reference, KIND).unwrap();
        let metadata = serde_json::to_vec(&stored).unwrap();
        let old: LegacyStored = serde_json::from_slice(&metadata).unwrap();
        assert_eq!(metadata, serde_json::to_vec(&old).unwrap());
        let legacy = codec::write(
            root.path(),
            "legacy-context",
            KIND,
            &old,
            Atoms { atoms: atoms.atoms },
            symbols,
        )
        .unwrap();
        assert_eq!(
            std::fs::read(reference.resolve(root.path()).unwrap()).unwrap(),
            std::fs::read(legacy.resolve(root.path()).unwrap()).unwrap()
        );
        assert!(
            read(root.path(), &legacy, u64::MAX)
                .unwrap()
                .represented
                .is_none()
        );
    }

    #[test]
    fn legacy_represented_parametric_configuration_recovers_policy_from_request_only() {
        use crate::parametric::*;
        use symbolica::symbol;
        let root = tempfile::tempdir().unwrap();
        let (x, eps, t) = symbol!(
            "legacy_represented::x",
            "legacy_represented::eps",
            "legacy_represented::t"
        );
        let input = Arc::new(
            ParametricIntegrand::new(
                vec![x],
                eps,
                ParametricDomain::UnitCube,
                vec![ParametricTerm::new(
                    Atom::one(),
                    vec![Atom::Zero],
                    vec![
                        PolynomialFactor::new(
                            Atom::var(x) - Atom::num(0.25),
                            -Atom::var(eps),
                            FactorRole::Singularity,
                        )
                        .with_semantics(FactorSemantics::Causal),
                    ],
                )],
            )
            .unwrap(),
        );
        let mut options = PreparationOptions::new(t);
        options.represented = Some((
            represented::NumericalMeaning::RepresentedValues,
            Default::default(),
        ));
        let prepared = prepare(input, options.clone(), root.path(), |_| {
            ControlFlow::Continue(())
        })
        .unwrap();
        let mut prior = prepared.receipt;
        let (mut metadata, atoms, symbols): (serde_json::Value, _, _) =
            codec::read(root.path(), &prior.configuration, KIND).unwrap();
        metadata.as_object_mut().unwrap().remove("represented");
        let legacy: LegacyStored = serde_json::from_value(metadata).unwrap();
        prior.configuration = codec::write(
            root.path(),
            "legacy-represented-context",
            KIND,
            &legacy,
            Atoms { atoms: atoms.atoms },
            symbols,
        )
        .unwrap();
        assert!(
            read(root.path(), &prior.configuration, u64::MAX)
                .unwrap()
                .represented
                .is_none()
        );
        let mut stages = Vec::new();
        let replay = resume(
            root.path(),
            &prior,
            "legacy-replay",
            &prior.publication.source_identity,
            &prior.publication.prepared_identity,
            32 * 1024 * 1024,
            |p| {
                stages.push(p.stage);
                ControlFlow::Continue(())
            },
        )
        .unwrap();
        assert!(!stages.contains(&Stage::Solve));
        assert!(stages.contains(&Stage::Verify));
        assert_eq!(
            replay.receipt.publication.source_identity,
            prior.publication.source_identity
        );
        assert_eq!(
            replay.receipt.publication.prepared_identity,
            prior.publication.prepared_identity
        );
        let request = gcad::staging::StagedRequest::read(
            root.path(),
            prior.evidence.request.as_ref().unwrap(),
            32 * 1024 * 1024,
        )
        .unwrap();
        let owner = request
            .request()
            .represented_input()
            .expect("original represented authority retained");
        assert_eq!(Some((owner.meaning(), owner.limits())), options.represented);
        options.represented.as_mut().unwrap().1.converted_literals += 1;
        prior.configuration = write(root.path(), &options).unwrap();
        assert!(matches!(
            resume(
                root.path(),
                &prior,
                "tampered-policy",
                &prior.publication.source_identity,
                &prior.publication.prepared_identity,
                32 * 1024 * 1024,
                |_| ControlFlow::Continue(())
            ),
            Err(Error::Association("represented policy/configuration"))
        ));
    }

    #[test]
    fn represented_configuration_version_is_explicit_and_unknown_version_refuses() {
        let root = tempfile::tempdir().unwrap();
        let mut options =
            PreparationOptions::new(symbolica::symbol!("represented_configuration::t"));
        options.represented = Some((
            represented::NumericalMeaning::RepresentedValues,
            Default::default(),
        ));
        let reference = write(root.path(), &options).unwrap();
        assert_eq!(
            read(root.path(), &reference, u64::MAX).unwrap().represented,
            options.represented
        );
        let (mut stored, atoms, symbols): (Stored, _, _) =
            codec::read(root.path(), &reference, KIND).unwrap();
        assert!(
            serde_json::from_slice::<LegacyStored>(&serde_json::to_vec(&stored).unwrap()).is_err()
        );
        stored.represented.as_mut().unwrap().version = 2;
        let foreign = codec::write(
            root.path(),
            "unknown-version",
            KIND,
            &stored,
            Atoms { atoms: atoms.atoms },
            symbols,
        )
        .unwrap();
        assert!(matches!(
            read(root.path(), &foreign, u64::MAX),
            Err(Error::Association(_))
        ));
    }

    #[test]
    fn malformed_binding_count_refuses_without_overflow() {
        let root = tempfile::tempdir().unwrap();
        let options = PreparationOptions::new(symbolica::symbol!("configuration_overflow::t"));
        let reference = write(root.path(), &options).unwrap();
        let (mut data, atoms, symbols): (Stored, _, _) =
            codec::read(root.path(), &reference, KIND).unwrap();
        data.bindings = usize::MAX;
        let forged = codec::write(
            root.path(),
            "forged-configuration",
            KIND,
            &data,
            Atoms { atoms: atoms.atoms },
            symbols,
        )
        .unwrap();
        assert!(matches!(
            read(root.path(), &forged, u64::MAX),
            Err(Error::Association(_))
        ));
    }
}
