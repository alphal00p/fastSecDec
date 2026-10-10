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
        represented: None,
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
