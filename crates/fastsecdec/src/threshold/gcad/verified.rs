use super::{GcadError, GcadRequest, RequestIdentity, Result, SignedFactor};
use symgcad::{
    output::{Cell, SolveResult, Status},
    verify::Verification,
};

/// Untrusted native solver output plus its complete typed request association.
/// Restoring these objects never constructs a verified decomposition directly.
#[derive(Clone, Debug)]
pub struct NativeDecomposition {
    identity: RequestIdentity,
    result: SolveResult,
}

impl NativeDecomposition {
    pub fn from_parts(identity: RequestIdentity, result: SolveResult) -> Self {
        Self { identity, result }
    }
    pub fn identity(&self) -> &RequestIdentity {
        &self.identity
    }
    pub fn result(&self) -> &SolveResult {
        &self.result
    }
    pub fn into_parts(self) -> (RequestIdentity, SolveResult) {
        (self.identity, self.result)
    }
}

/// Independently verified generic open cells for one exact native request.
/// No endpoint, auxiliary-regulator, projective or full-integral claim follows.
#[derive(Debug)]
pub struct VerifiedDecomposition {
    request: GcadRequest,
    result: SolveResult,
    verification: Verification,
}

impl GcadRequest {
    /// Synchronous native solve. The caller owns scheduling and hard resource
    /// bounds; native cooperative checks cannot interrupt every CAS operation.
    pub fn solve(&self) -> Result<NativeDecomposition> {
        let result =
            symgcad::solver::solve(self.problem()).map_err(|e| GcadError::Solve(e.to_string()))?;
        Ok(NativeDecomposition::from_parts(
            self.identity().clone(),
            result,
        ))
    }
    pub fn verify(&self, evidence: NativeDecomposition) -> Result<VerifiedDecomposition> {
        if self.identity() != &evidence.identity
            || !self.matches_native_problem(&evidence.result.problem)
        {
            return Err(GcadError::RequestMismatch);
        }
        if evidence.result.status != Status::CompleteGeneric {
            return Err(GcadError::Incomplete(evidence.result.status));
        }
        if !self.preserves_parameter_fibers(&evidence.result.order) {
            return Err(GcadError::Verification(
                "native lifting order changes symbol roles or mixes parameter and integration directions".into(),
            ));
        }
        let verification = symgcad::verify::verify(&evidence.result)
            .map_err(|e| GcadError::Verification(e.to_string()))?;
        if !verification.verified {
            return Err(GcadError::Verification(
                "native verifier declined completeness".into(),
            ));
        }
        // A declared positive factor must be proved positive on every retained
        // cell. Adding it as a constraint would silently discard input regions.
        if evidence.result.cells.iter().any(|cell| {
            self.signed_factors().iter().any(|f| {
                f.semantics == crate::parametric::FactorSemantics::Positive
                    && cell.signs[f.split_index] != 1
            })
        }) {
            return Err(GcadError::Unsupported(
                "a declared positive factor is nonpositive on an admitted cell".into(),
            ));
        }
        Ok(VerifiedDecomposition {
            request: self.clone(),
            result: evidence.result,
            verification,
        })
    }
    /// Convenience for callers that do not need unsuccessful native evidence.
    /// Generation staging should call [`Self::solve`], persist its raw receipt,
    /// then [`Self::verify`], so an incomplete/resource-limited solve is retained.
    pub fn solve_verified(&self) -> Result<VerifiedDecomposition> {
        self.verify(self.solve()?)
    }
}

impl VerifiedDecomposition {
    pub fn request(&self) -> &GcadRequest {
        &self.request
    }
    pub fn native_result(&self) -> &SolveResult {
        &self.result
    }
    pub fn verification(&self) -> &Verification {
        &self.verification
    }
    pub fn cells(&self) -> impl ExactSizeIterator<Item = VerifiedCell<'_>> {
        self.result
            .cells
            .iter()
            .enumerate()
            .map(|(index, cell)| VerifiedCell {
                decomposition: self,
                index,
                cell,
            })
    }
}

/// Immutable raw-cell view tied to the verified owner's lifetime. Native root
/// selectors and exceptional polynomial IDs remain in that owner's result.
#[derive(Clone, Copy, Debug)]
pub struct VerifiedCell<'a> {
    decomposition: &'a VerifiedDecomposition,
    index: usize,
    cell: &'a Cell,
}
impl<'a> VerifiedCell<'a> {
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn native(&self) -> &'a Cell {
        self.cell
    }
    pub fn decomposition(&self) -> &'a VerifiedDecomposition {
        self.decomposition
    }
    /// Return the sign associated with an original physical term/factor.
    pub fn signed_factors(&self) -> impl ExactSizeIterator<Item = (&'a SignedFactor, i8)> {
        self.decomposition
            .request
            .signed_factors()
            .iter()
            .map(|f| (f, self.cell.signs[f.split_index]))
    }
}
