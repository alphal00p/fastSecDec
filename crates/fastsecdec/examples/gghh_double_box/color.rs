//! Explicit SU(3) color closure through the shared HEPKit/Idenso owner.

use idenso::{
    color::{CS, ColorSimplifier, ColorSimplifySettings},
    tensor::{AlgebraContraction, AlgebraSettings, ReductionStatus, SymbolicTensor},
};
use symbolica::atom::{Atom, AtomCore};

use super::Result;

pub fn contract(numerator: &Atom, projector: &Atom) -> Result<Atom> {
    let source = SymbolicTensor::infer(numerator * projector)?;
    let settings = AlgebraSettings {
        color: Some(ColorSimplifySettings::default()),
        contract: AlgebraContraction::None,
        ..AlgebraSettings::default()
    };
    // GammaLoop uses this same native option for physical evaluator inputs.
    // Gamma/epsilon identities and independent Lorentz contractions stay off.
    let symbolic = source.simplify_algebra(&settings)?;
    let explicit = source.simplify_algebra(&AlgebraSettings {
        color: Some(ColorSimplifySettings::default().with_cof_dimension_invariants()),
        ..settings
    })?;
    if symbolic.reduction_status() != ReductionStatus::Complete
        || explicit.reduction_status() != ReductionStatus::Complete
    {
        return Err("native ggHH color reduction did not complete".into());
    }
    let reduced = explicit.expression();
    if symbolic.expression().to_cof_dimension_invariants() != *reduced {
        return Err("native symbolic and explicit color policies disagree".into());
    }
    if [
        CS.cas,
        CS.idx,
        CS.gram,
        CS.f,
        CS.t,
        CS.d,
        CS.fundamental_rep,
        CS.adjoint_rep,
    ]
    .iter()
    .any(|&symbol| reduced.contains_symbol(symbol))
    {
        return Err("ggHH color projection retained an unresolved color object".into());
    }
    Ok(reduced.clone())
}
