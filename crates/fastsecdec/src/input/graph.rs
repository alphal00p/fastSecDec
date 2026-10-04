use std::{collections::BTreeMap, sync::Arc};

use feynkit_graph::{EdgeId, FeynmanDiagram, IntegralFamily};
use feynkit_kinematics::Kinematics;
use feynkit_model::Model;
use idenso::tensor::AlgebraSettings;
use symbolica::atom::Atom;

use crate::{Error, Result, input::contract_numerator};

/// A native diagram plus the integral data absent from HEPKit's graph type.
///
/// Propagator powers follow HEPKit's ascending internal-edge order. The graph
/// remains the authoritative owner of topology, routing and numerator fragments.
#[derive(Clone, Debug)]
pub struct GraphIntegral {
    diagram: Arc<FeynmanDiagram>,
    family: IntegralFamily,
    propagator_edges: Vec<EdgeId>,
    powers: Vec<u32>,
    measure_multiplier: Atom,
}

impl GraphIntegral {
    pub fn new(diagram: Arc<FeynmanDiagram>, kinematics: &Kinematics) -> Result<Self> {
        diagram.validate()?;
        super::validation::validate_denominators(&diagram)?;
        let family = diagram.propagator_family(kinematics)?;
        let propagator_edges: Vec<_> = diagram
            .edges()
            .filter(|(_, endpoints, edge)| {
                endpoints.source.is_some()
                    && endpoints.target.is_some()
                    && edge.external.is_none()
                    && !edge.is_dummy
            })
            .map(|(id, _, _)| id)
            .collect();
        debug_assert_eq!(propagator_edges.len(), family.denominators().len());
        let powers = vec![1; propagator_edges.len()];
        Ok(Self {
            diagram,
            family,
            propagator_edges,
            powers,
            measure_multiplier: Atom::one(),
        })
    }

    /// Parse either native compact HEPKit DOT or its stable exported dialect.
    pub fn from_dot(model: Arc<Model>, dot: &str, kinematics: &Kinematics) -> Result<Self> {
        Self::new(Arc::new(FeynmanDiagram::from_dot(model, dot)?), kinematics)
    }

    /// Replace selected propagator powers using the graph's stable edge IDs.
    pub fn with_powers(mut self, powers: &BTreeMap<EdgeId, u32>) -> Result<Self> {
        for (&edge, &power) in powers {
            let position = self
                .propagator_edges
                .iter()
                .position(|candidate| *candidate == edge)
                .ok_or(Error::UnknownPropagator(edge))?;
            if power == 0 {
                return Err(Error::InvalidPower { edge, power });
            }
            self.powers[position] = power;
        }
        Ok(self)
    }

    /// Extra normalization, e.g. an explicit scale factor, beyond the normalized
    /// Minkowski measure `prod_l d^D k_l / (i*pi^(D/2))`.
    pub fn with_measure_multiplier(mut self, multiplier: Atom) -> Self {
        self.measure_multiplier = multiplier;
        self
    }

    pub fn diagram(&self) -> &Arc<FeynmanDiagram> {
        &self.diagram
    }

    pub fn family(&self) -> &IntegralFamily {
        &self.family
    }

    pub fn propagator_edges(&self) -> &[EdgeId] {
        &self.propagator_edges
    }

    pub fn powers(&self) -> &[u32] {
        &self.powers
    }

    pub fn measure_multiplier(&self) -> &Atom {
        &self.measure_multiplier
    }

    /// Contract the diagram's numerator with its projector using existing
    /// Idenso identities, retaining the scalar prefactor and overall weight.
    /// The diagnostic automorphism order is never applied a second time.
    pub fn scalar_numerator(&self, settings: &AlgebraSettings) -> Result<Atom> {
        contract_numerator(&self.diagram, self.family.kinematics(), settings)
    }
}
