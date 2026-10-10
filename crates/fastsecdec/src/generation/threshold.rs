//! Internal reuse of the symbolic subtraction engine for certified threshold
//! inputs. This low-level tuple entry itself carries no geometry certificate.
use super::{
    EndpointProfileRow, GenerationError, GenerationMode, GenerationOptions, mapping::MappedTerm,
    subtraction::subtract_profiled_with_regulators,
};
use symbolica::atom::{Atom, Symbol};

pub(crate) struct Continued {
    pub expression: Atom,
    pub endpoint_profiles: Vec<EndpointProfileRow>,
}

pub(crate) fn subtract(
    terms: Vec<(Atom, Atom, Vec<Atom>)>,
    coordinates: &[Symbol],
    regulators: &[Symbol],
    options: &GenerationOptions,
) -> Result<Continued, GenerationError> {
    if options.mode != GenerationMode::Symbolic
        || options.contour_enabled()
        || options.contour_jacobian != crate::contour::ContourJacobian::Symbolic
        || options.source_sectors.is_some()
    {
        return Err(GenerationError::Invariant(
            "threshold continuation requires symbolic endpoints, no contour and the complete admitted cell family".into(),
        ));
    }
    let output = subtract_profiled_with_regulators(
        terms
            .into_iter()
            .map(|(prefactor, regular, powers)| MappedTerm {
                prefactor,
                regular,
                powers,
            })
            .collect(),
        coordinates,
        regulators,
        options,
    )?;
    Ok(Continued {
        expression: output.expression,
        endpoint_profiles: output.endpoint_profiles,
    })
}
