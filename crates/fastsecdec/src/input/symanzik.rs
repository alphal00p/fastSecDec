//! Numerator-independent access to the native graph family's exact U and F.

use super::GraphIntegral;
use crate::{Atom, EdgeId, Error, Result};

/// Pre-sector projective polynomials in native propagator order.
///
/// This is algebraic input for downstream tools, not a sector decomposition,
/// domain certificate, or no-deformation integration algorithm. Numerators,
/// powers and measure factors are retained by `GraphIntegral`; they do not
/// enter U/F and are never contracted by this interface.
#[derive(Clone, Debug)]
pub struct GraphSymanzik {
    parameters: Vec<Atom>,
    propagator_edges: Vec<EdgeId>,
    loop_count: usize,
    u: Atom,
    f: Atom,
}

impl GraphSymanzik {
    pub fn from_graph(graph: &GraphIntegral, parameters: Vec<Atom>) -> Result<Self> {
        if parameters.len() != graph.propagator_edges().len() {
            return Err(Error::ParameterCount {
                expected: graph.propagator_edges().len(),
                actual: parameters.len(),
            });
        }
        // Native IntegralFamily validates distinct formal labels and their
        // absence from the admitted family, and owns the determinant algebra.
        let (u, f) = graph.family().symanzik(&parameters)?;
        if u.is_zero() {
            return Err(Error::SingularLoopForm);
        }
        Ok(Self {
            parameters,
            propagator_edges: graph.propagator_edges().to_vec(),
            loop_count: graph.family().loop_momenta().len(),
            u,
            f,
        })
    }

    pub fn parameters(&self) -> &[Atom] {
        &self.parameters
    }
    pub fn propagator_edges(&self) -> &[EdgeId] {
        &self.propagator_edges
    }
    pub fn loop_count(&self) -> usize {
        self.loop_count
    }
    pub fn u(&self) -> &Atom {
        &self.u
    }
    /// May be identically zero; downstream admission must handle that explicitly.
    pub fn f(&self) -> &Atom {
        &self.f
    }
}
