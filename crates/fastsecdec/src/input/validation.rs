//! Admission checks for information the native quadratic family intentionally
//! does not infer: widths and custom UFO denominator templates.

use feynkit_graph::FeynmanDiagram;
use feynkit_model::ParameterType;
use std::collections::{BTreeMap, HashMap};
use symbolica::{
    atom::{Atom, AtomCore, Symbol},
    domains::float::{Complex, Float, SingleFloat},
    function, symbol,
};

use crate::{Error, Result};

pub(super) fn validate_denominators(diagram: &FeynmanDiagram) -> Result<()> {
    let model = diagram.model();
    // This is the existing UFO quadratic template used by native HEPKit models,
    // not a second parser or propagator builder. Symbolica verifies equality.
    let momentum = function!(symbol!("UFO::P"), function!(symbol!("UFO::idx"), 1, 1));
    for (edge_id, endpoints, edge) in diagram.edges() {
        if endpoints.source.is_none()
            || endpoints.target.is_none()
            || edge.external.is_some()
            || edge.is_dummy
        {
            continue;
        }
        let particle = model.particle_by_id(edge.particle)?;
        let width = model.particle_width(edge.particle)?;
        let zero_width = match width.value {
            Some(value) => value.re == 0.0 && value.im == 0.0,
            None => width.expression.as_ref().is_some_and(Atom::is_zero),
        };
        if !zero_width {
            return Err(Error::UnsupportedWidth {
                edge: edge_id,
                parameter: width.name.clone(),
            });
        }
        let mass = model.particle_mass(edge.particle)?;
        let real_mass = match mass.value {
            Some(value) => value.re.is_finite() && value.im == 0.0,
            None => mass.parameter_type == ParameterType::Real,
        };
        if !real_mass {
            return Err(Error::UnsupportedMass {
                edge: edge_id,
                parameter: mass.name.clone(),
            });
        }
        if let Some((propagator_id, _)) = model.particle_propagator(particle)? {
            let propagator = model.propagator_by_id(propagator_id)?;
            let quadratic = momentum.pow(2) - particle.symbolic_mass(model).pow(2);
            if !(&propagator.denominator - quadratic).expand().is_zero() {
                return Err(Error::UnsupportedDenominator {
                    edge: edge_id,
                    propagator: propagator.name.clone(),
                });
            }
        }
    }
    Ok(())
}

/// Recheck overrides that are applied after the native model was admitted.
/// Widths never enter the quadratic family, so they must not be silently lost.
pub(super) fn validate_scalar_bindings(
    diagram: &FeynmanDiagram,
    values: &BTreeMap<Symbol, Atom>,
) -> Result<()> {
    let model = diagram.model();
    for (edge_id, endpoints, edge) in diagram.edges() {
        if endpoints.source.is_none()
            || endpoints.target.is_none()
            || edge.external.is_some()
            || edge.is_dummy
        {
            continue;
        }
        let width = model.particle_width(edge.particle)?;
        if let Some(value) = values.get(&symbol!(&format!("UFO::{}", width.name)))
            && !value.is_zero()
        {
            return Err(Error::UnsupportedWidth {
                edge: edge_id,
                parameter: width.name.clone(),
            });
        }
        let mass = model.particle_mass(edge.particle)?;
        if let Some(value) = values.get(&symbol!(&format!("UFO::{}", mass.name))) {
            let numerical = value.evaluate_with_prec(&HashMap::<Atom, Complex<Float>>::new(), 128);
            if !value.is_real().is_true()
                || !numerical.is_ok_and(|number| number.re.is_finite() && number.im.is_zero())
            {
                return Err(Error::UnsupportedMass {
                    edge: edge_id,
                    parameter: mass.name.clone(),
                });
            }
        }
    }
    Ok(())
}
