use feynkit_graph::FeynmanDiagram;
use feynkit_kinematics::Kinematics;
use idenso::tensor::{AlgebraContraction, AlgebraSettings, SymbolicTensor};
use spenso::{network::parsing::AtomStructureExt, structure::TensorStructure};
use symbolica::atom::Atom;

use crate::{Error, Result};

/// Use HEPKit's established identity families while retaining unrelated scalar
/// factorization. No vacuum tensor averaging or external Gram inverse is used.
pub fn default_algebra_settings() -> AlgebraSettings {
    AlgebraSettings {
        contract: AlgebraContraction::Minimal,
        ..AlgebraSettings::hep()
    }
}

pub fn contract_numerator(
    diagram: &FeynmanDiagram,
    kinematics: &Kinematics,
    settings: &AlgebraSettings,
) -> Result<Atom> {
    let dimension = kinematics.dimension().to_symbolic();
    let projected = (diagram.numerator()
        * diagram.projector()
        * diagram.numerator_prefactor()
        * diagram.overall_factor())
    .with_lorentz_dimension(dimension.as_view());
    let routed = diagram.loop_momentum_basis().route_expression(&projected);
    let tensor = SymbolicTensor::infer(routed)
        .and_then(|tensor| tensor.simplify_algebra(settings))
        .and_then(|tensor| tensor.to_dots())
        .map_err(|error| Error::Tensor(error.to_string()))?;
    if !tensor.structure().canonical().is_scalar() {
        return Err(Error::FreeTensorIndices);
    }
    Ok(kinematics.apply(tensor.expression()))
}
