//! Recipe-neutral native residual records, before density combination or faces.
use super::*;
use crate::generation::{mapping::PreparedTerm, numerical_dual::chart::OpaqueMapping};

pub(in crate::generation::streaming) struct PreparedData {
    pub index: usize,
    pub source_identity: String,
    pub map_reference: RecordRef,
    pub map: SectorMap,
    pub parameters: Vec<Symbol>,
    pub retains_declarations: bool,
    pub terms: Option<Vec<PreparedTerm>>,
    pub opaque: Option<OpaqueMapping>,
}

#[derive(Serialize, Deserialize)]
struct ResidualTerm {
    prefactor: usize,
    powers: Vec<usize>,
    residuals: Vec<(usize, usize, FactorSemantics)>,
}

type StoredOpaque = (Vec<Mapped>, Vec<Vec<DeferredFactor>>);
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Prepared {
    index: usize,
    source_identity: String,
    map_reference: RecordRef,
    map: Map,
    dimension: usize,
    retains_declarations: bool,
    terms: Option<Vec<ResidualTerm>>,
    opaque: Option<StoredOpaque>,
}

pub(in crate::generation::streaming) fn write_prepared(
    root: &Path,
    data: &PreparedData,
) -> Result<RecordRef, StreamingError> {
    let mut atoms = Atoms::default();
    let terms = data.terms.as_ref().map(|terms| {
        terms
            .iter()
            .map(|term| ResidualTerm {
                prefactor: atoms.push(&term.prefactor),
                powers: term.powers.iter().map(|power| atoms.push(power)).collect(),
                residuals: term
                    .residuals
                    .iter()
                    .map(|(residual, exponent, semantics)| {
                        (atoms.push(residual), atoms.push(exponent), *semantics)
                    })
                    .collect(),
            })
            .collect()
    });
    let opaque = data.opaque.as_ref().map(|(mapped, terms)| {
        let mapped = mapped
            .iter()
            .map(|term| Mapped {
                prefactor: atoms.push(&term.prefactor),
                powers: term.powers.iter().map(|power| atoms.push(power)).collect(),
                regular: atoms.push(&term.regular),
            })
            .collect();
        let terms = terms
            .iter()
            .map(|term| {
                term.factors
                    .iter()
                    .map(|factor| DeferredFactor {
                        polynomial: atoms.push(&factor.polynomial),
                        exponent: atoms.push(&factor.exponent),
                        valuation: factor.valuation.clone(),
                    })
                    .collect()
            })
            .collect();
        (mapped, terms)
    });
    codec::write(
        root,
        &format!("prepared-source-{}", data.index),
        "prepared-chart-source-v1",
        &Prepared {
            index: data.index,
            source_identity: data.source_identity.clone(),
            map_reference: data.map_reference.clone(),
            map: (&data.map).into(),
            dimension: data.parameters.len(),
            retains_declarations: data.retains_declarations,
            terms,
            opaque,
        },
        atoms,
        data.parameters.clone(),
    )
}

pub(in crate::generation::streaming) fn read_prepared(
    root: &Path,
    reference: &RecordRef,
) -> Result<PreparedData, StreamingError> {
    let (stored, atoms, parameters): (Prepared, _, _) =
        codec::read(root, reference, "prepared-chart-source-v1")?;
    let map = stored.map.native()?;
    if parameters.len() != stored.dimension || map.dimension() != stored.dimension {
        return Err(invalid("prepared source dimension mismatch"));
    }
    let terms = stored
        .terms
        .map(|terms| {
            terms
                .into_iter()
                .map(|term| {
                    if term.powers.len() != stored.dimension {
                        return Err(invalid("prepared source endpoint dimension mismatch"));
                    }
                    Ok(PreparedTerm {
                        prefactor: atoms.take(term.prefactor)?,
                        powers: term
                            .powers
                            .into_iter()
                            .map(|index| atoms.take(index))
                            .collect::<Result<_, _>>()?,
                        residuals: term
                            .residuals
                            .into_iter()
                            .map(|(residual, exponent, semantics)| {
                                Ok((atoms.take(residual)?, atoms.take(exponent)?, semantics))
                            })
                            .collect::<Result<_, StreamingError>>()?,
                    })
                })
                .collect::<Result<_, StreamingError>>()
        })
        .transpose()?;
    let opaque = stored
        .opaque
        .map(|(mapped, terms)| {
            if mapped.len() != terms.len() {
                return Err(invalid("prepared opaque source term count mismatch"));
            }
            let mapped = mapped
                .into_iter()
                .map(|term| {
                    if term.powers.len() != stored.dimension {
                        return Err(invalid("prepared opaque endpoint dimension mismatch"));
                    }
                    Ok(MappedTerm {
                        prefactor: atoms.take(term.prefactor)?,
                        regular: atoms.take(term.regular)?,
                        powers: term
                            .powers
                            .into_iter()
                            .map(|index| atoms.take(index))
                            .collect::<Result<_, _>>()?,
                    })
                })
                .collect::<Result<_, StreamingError>>()?;
            let terms = terms
                .into_iter()
                .map(|factors| {
                    Ok(DualTerm {
                        factors: factors
                            .into_iter()
                            .map(|factor| {
                                if factor.valuation.len() != stored.dimension {
                                    return Err(invalid(
                                        "prepared opaque valuation dimension mismatch",
                                    ));
                                }
                                Ok(DualFactor {
                                    polynomial: atoms.take(factor.polynomial)?,
                                    exponent: atoms.take(factor.exponent)?,
                                    valuation: factor.valuation,
                                })
                            })
                            .collect::<Result<_, StreamingError>>()?,
                    })
                })
                .collect::<Result<_, StreamingError>>()?;
            Ok((mapped, terms))
        })
        .transpose()?;
    Ok(PreparedData {
        index: stored.index,
        source_identity: stored.source_identity,
        map_reference: stored.map_reference,
        map,
        parameters,
        retains_declarations: stored.retains_declarations,
        terms,
        opaque,
    })
}
