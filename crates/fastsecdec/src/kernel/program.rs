//! Native symbolic-to-numeric boundary. One exact program owns every numeric path.
#[cfg(test)]
mod captured;
use super::{KernelError, cancellation::Cancellation};
use symbolica::{
    atom::{AliasedAtom, Atom, AtomCore, AtomView, Symbol},
    domains::{float::Complex, rational::Rational},
    evaluate::ExpressionEvaluator,
};

pub(super) type ExactProgram = ExpressionEvaluator<Complex<Rational>>;

pub(super) struct SectorProgram {
    pub parameters: Vec<Symbol>,
    pub exact: ExactProgram,
    pub cancellation: Cancellation,
    pub exact_zero: Vec<bool>,
    pub real_coefficients: Vec<bool>,
}

/// Coordinate-image maps are flat and shared within one generated vector.
/// Empty maps are allowed for exact zero padding or plain legacy expressions.
pub(super) fn build(
    parameters: Vec<Symbol>,
    coefficients: &[AliasedAtom],
    cancellation: Cancellation,
) -> Result<SectorProgram, KernelError> {
    let aliases = coefficients
        .iter()
        .map(AliasedAtom::get_aliases)
        .find(|aliases| !aliases.is_empty());
    if let Some(aliases) = aliases
        && coefficients.iter().any(|coefficient| {
            !coefficient.get_aliases().is_empty() && coefficient.get_aliases() != aliases
        })
    {
        return Err(KernelError::Compilation(
            "coefficient vector has inconsistent native alias definitions".into(),
        ));
    }
    let roots = coefficients
        .iter()
        .map(AliasedAtom::get_root)
        .collect::<Vec<_>>();
    let variables = parameters.iter().map(|p| Atom::var(*p)).collect::<Vec<_>>();
    let mut builder = Atom::evaluator_multiple(&roots, &variables)
        .direct_translation(true)
        .horner_iterations(0);
    if let Some(aliases) = aliases {
        // Register a shared map once; native AliasedAtom::evaluator_multiple
        // would register identical definitions again for each coefficient.
        let mut ordered = aliases.iter().collect::<Vec<_>>();
        ordered.sort_by(|a, b| a.0.cmp(b.0));
        builder = builder
            .add_aliases(
                ordered
                    .into_iter()
                    .map(|(handle, body)| (handle.clone(), body.clone())),
            )
            .map_err(|error| KernelError::Compilation(error.to_string()))?;
    }
    let exact = builder
        .build()
        .map_err(|error| KernelError::Compilation(error.to_string()))?;
    Ok(SectorProgram {
        parameters,
        exact,
        cancellation,
        exact_zero: roots.iter().map(|root| root.is_zero()).collect(),
        real_coefficients: coefficients.iter().map(is_real).collect(),
    })
}

/// Native literal coefficients plus the certified real phase-one domain provide
/// this sufficient test. Only definitions used by the root contribute.
pub(super) fn is_real(coefficient: &AliasedAtom) -> bool {
    let symbols = coefficient.get_root().get_all_symbols(true);
    !super::has_complex_coefficients(coefficient.get_root())
        && coefficient.get_aliases().iter().all(|(handle, body)| {
            let used = match handle.as_view() {
                AtomView::Var(handle) => symbols.contains(&handle.get_symbol()),
                _ => true,
            };
            !used || !super::has_complex_coefficients(body)
        })
}

pub(super) fn encode(program: &ExactProgram) -> Result<Vec<u8>, KernelError> {
    bincode::serde::encode_to_vec(program, bincode::config::standard())
        .map_err(|error| KernelError::Artifact(format!("native evaluator encoding: {error}")))
}

pub(super) fn decode(bytes: &[u8]) -> Result<ExactProgram, KernelError> {
    // Native callbacks for external fixed Gamma/polygamma constants must exist
    // before the portable program imports their symbol identities.
    let _ = symbolica::transcendental::gamma();
    let (program, consumed) =
        bincode::serde::borrow_decode_from_slice(bytes, bincode::config::standard()).map_err(
            |error| KernelError::Artifact(format!("native evaluator decoding: {error}")),
        )?;
    if consumed != bytes.len() {
        return Err(KernelError::Artifact(
            "trailing native evaluator bytes".into(),
        ));
    }
    Ok(program)
}
