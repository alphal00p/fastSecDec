//! Native request reconstruction; no solving and no trusted proof decoding.
use super::{Error, RequestRecord, Result};
use crate::{
    generation::{
        identity::{CanonicalAtom, CanonicalSymbol},
        streaming::{
            codec::{self, Atoms, invalid},
            input::Input,
        },
    },
    parametric::FactorRole,
    threshold::{
        gcad::{AliasRole, DomainOrigin, GcadKinematics, GcadRequest, PreparedDomain, Problem},
        projective::{AffineProjectivePreparation, FactorOrigin},
    },
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
use symbolica::{
    atom::{Atom, AtomCore},
    prelude::Rational,
};

const KIND: &str = "threshold-gcad-request-v1";

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Origin {
    UnitCube,
    AffineProjective { eliminated_index: usize },
    ExplicitPrepared { provenance: String },
}
impl From<&DomainOrigin> for Origin {
    fn from(origin: &DomainOrigin) -> Self {
        match origin {
            DomainOrigin::NativeUnitCube => Self::UnitCube,
            DomainOrigin::AffineProjective { eliminated_index } => Self::AffineProjective {
                eliminated_index: *eliminated_index,
            },
            DomainOrigin::ExplicitPrepared { provenance } => Self::ExplicitPrepared {
                provenance: provenance.clone(),
            },
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredRequest {
    source_identity: String,
    witness: String,
    input: Input,
    origin: Origin,
    coordinates: Vec<usize>,
    domain_positive: Vec<usize>,
    exact_values: Vec<(usize, usize)>,
    runtime_parameters: Vec<usize>,
    parameter_positive: Vec<usize>,
    problem: Problem,
}

fn origin(origin: FactorOrigin) -> (u8, usize) {
    match origin {
        FactorOrigin::OriginalFactor { factor_index } => (0, factor_index),
        FactorOrigin::OriginalMonomial { parameter_index } => (1, parameter_index),
    }
}

/// A canonical association witness, not an alternate symbolic representation.
/// Native canonical serializers feed a streaming hash, without building a
/// second transformed density or expanding any factor.
fn witness(request: &GcadRequest, source_identity: &str) -> Result<String> {
    let mut hash = blake3::Hasher::new();
    hash.update(b"fastsecdec-threshold-gcad-request-association-v1\0");
    fn item<T: Serialize>(hash: &mut blake3::Hasher, value: &T) -> Result<()> {
        serde_json::to_writer(&mut *hash, value).map_err(invalid)?;
        hash.update(b"\0");
        Ok(())
    }
    item(&mut hash, &source_identity)?;
    item(&mut hash, &Origin::from(request.domain().origin()))?;
    item(
        &mut hash,
        &request
            .domain()
            .coordinates()
            .iter()
            .copied()
            .map(CanonicalSymbol)
            .collect::<Vec<_>>(),
    )?;
    item(
        &mut hash,
        &request
            .domain()
            .strict_positive()
            .iter()
            .map(CanonicalAtom)
            .collect::<Vec<_>>(),
    )?;
    item(
        &mut hash,
        &request
            .kinematics()
            .runtime_parameters
            .iter()
            .copied()
            .map(CanonicalSymbol)
            .collect::<Vec<_>>(),
    )?;
    let mut exact_values = request.kinematics().exact_values.iter().collect::<Vec<_>>();
    exact_values.sort_by_cached_key(|(symbol, _)| Atom::var(**symbol).to_canonical_string());
    for (symbol, value) in exact_values {
        item(
            &mut hash,
            &(
                CanonicalSymbol(*symbol),
                CanonicalAtom(&Atom::num(value.clone())),
            ),
        )?;
    }
    item(
        &mut hash,
        &request
            .kinematics()
            .strict_positive
            .iter()
            .map(CanonicalAtom)
            .collect::<Vec<_>>(),
    )?;
    item(&mut hash, request.problem())?;
    for alias in request.aliases() {
        item(
            &mut hash,
            &(
                CanonicalSymbol(alias.symbol),
                &alias.name,
                alias.role == AliasRole::RuntimeParameter,
            ),
        )?;
    }
    for term in request.prepared_terms() {
        item(&mut hash, &CanonicalAtom(term.prefactor()))?;
        item(
            &mut hash,
            &term
                .monomial_powers()
                .iter()
                .map(CanonicalAtom)
                .collect::<Vec<_>>(),
        )?;
        item(&mut hash, &term.factors().len())?;
        for factor in term.factors() {
            item(
                &mut hash,
                &(
                    CanonicalAtom(factor.polynomial()),
                    CanonicalAtom(factor.exponent()),
                    factor.role() == FactorRole::Polynomial,
                    factor.semantics(),
                ),
            )?;
        }
    }
    if let Some(preparation) = request.projective_preparation() {
        item(
            &mut hash,
            &preparation
                .images()
                .iter()
                .map(CanonicalAtom)
                .collect::<Vec<_>>(),
        )?;
        item(&mut hash, &CanonicalAtom(preparation.measure_jacobian()))?;
        item(
            &mut hash,
            &preparation
                .factor_origins()
                .iter()
                .map(|terms| terms.iter().copied().map(origin).collect::<Vec<_>>())
                .collect::<Vec<_>>(),
        )?;
    }
    for factor in request.signed_factors() {
        item(
            &mut hash,
            &(
                factor.term_index,
                factor.factor_index,
                origin(factor.origin),
                factor.split_index,
                CanonicalAtom(&factor.original_polynomial),
                CanonicalAtom(&factor.prepared_polynomial),
                CanonicalAtom(&factor.exponent),
                factor.semantics,
            ),
        )?;
    }
    Ok(hash.finalize().to_hex().to_string())
}

pub(super) fn write(root: &Path, request: &GcadRequest) -> Result<RequestRecord> {
    let source_identity = crate::generation::source_identity(
        request.input(),
        &request.kinematics().runtime_parameters,
        &[],
    )?;
    let mut atoms = Atoms::default();
    let input = Input::encode(request.input(), &mut atoms);
    let mut symbols = request
        .input()
        .parameters()
        .iter()
        .copied()
        .chain([request.input().regulator()])
        .collect::<Vec<_>>();
    let mut symbol = |symbol| {
        let index = symbols.len();
        symbols.push(symbol);
        index
    };
    let stored = StoredRequest {
        source_identity: source_identity.clone(),
        witness: witness(request, &source_identity)?,
        input,
        origin: request.domain().origin().into(),
        coordinates: request
            .domain()
            .coordinates()
            .iter()
            .copied()
            .map(&mut symbol)
            .collect(),
        domain_positive: request
            .domain()
            .strict_positive()
            .iter()
            .map(|x| atoms.push(x))
            .collect(),
        exact_values: request
            .kinematics()
            .exact_values
            .iter()
            .map(|(s, r)| (symbol(*s), atoms.push(&Atom::num(r.clone()))))
            .collect(),
        runtime_parameters: request
            .kinematics()
            .runtime_parameters
            .iter()
            .copied()
            .map(&mut symbol)
            .collect(),
        parameter_positive: request
            .kinematics()
            .strict_positive
            .iter()
            .map(|x| atoms.push(x))
            .collect(),
        problem: request.problem().clone(),
    };
    let record = codec::write(root, "gcad-request", KIND, &stored, atoms, symbols)?;
    Ok(RequestRecord {
        source_identity,
        record,
    })
}

pub(super) fn read(root: &Path, receipt: &RequestRecord) -> Result<GcadRequest> {
    let (stored, atoms, symbols): (StoredRequest, _, _) = codec::read(root, &receipt.record, KIND)?;
    let symbol = |i: usize| {
        symbols
            .get(i)
            .copied()
            .ok_or_else(|| invalid("GCAD request Symbol index out of bounds"))
    };
    let prefix = stored
        .input
        .parameters
        .checked_add(1)
        .ok_or_else(|| invalid("GCAD input symbol count overflow"))?;
    let input_symbols = symbols
        .get(..prefix)
        .ok_or_else(|| invalid("GCAD input symbol layout"))?;
    let input = stored.input.decode(&atoms, input_symbols)?;
    let coordinates = stored
        .coordinates
        .into_iter()
        .map(symbol)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let positive = stored
        .domain_positive
        .into_iter()
        .map(|i| atoms.take(i))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut exact_values = BTreeMap::new();
    for (s, value) in stored.exact_values {
        let value = Rational::try_from(atoms.take(value)?.as_view()).map_err(invalid)?;
        if exact_values.insert(symbol(s)?, value).is_some() {
            return Err(invalid("duplicate GCAD exact kinematic binding").into());
        }
    }
    let kinematics = GcadKinematics {
        exact_values,
        runtime_parameters: stored
            .runtime_parameters
            .into_iter()
            .map(symbol)
            .collect::<std::result::Result<_, _>>()?,
        strict_positive: stored
            .parameter_positive
            .into_iter()
            .map(|i| atoms.take(i))
            .collect::<std::result::Result<_, _>>()?,
    };
    let request = match stored.origin {
        Origin::UnitCube => GcadRequest::unit_cube(
            &input,
            kinematics,
            stored.problem.solver.clone(),
            stored.problem.limits.clone(),
        )?,
        Origin::AffineProjective { eliminated_index } => {
            let preparation = AffineProjectivePreparation::eliminate(&input, eliminated_index)
                .map_err(invalid)?;
            GcadRequest::projective(
                preparation,
                kinematics,
                stored.problem.solver.clone(),
                stored.problem.limits.clone(),
            )?
        }
        Origin::ExplicitPrepared { provenance } => GcadRequest::prepared(
            &input,
            PreparedDomain::explicit(coordinates.clone(), positive.clone(), provenance)?,
            kinematics,
            stored.problem.solver.clone(),
            stored.problem.limits.clone(),
        )?,
    };
    let actual_source = crate::generation::source_identity(
        request.input(),
        &request.kinematics().runtime_parameters,
        &[],
    )?;
    if request.domain().coordinates() != coordinates
        || request.domain().strict_positive() != positive
        || !request.matches_native_problem(&stored.problem)
        || stored.source_identity != receipt.source_identity
        || stored.source_identity != actual_source
        || stored.witness != witness(&request, &actual_source)?
    {
        return Err(Error::Association);
    }
    Ok(request)
}
