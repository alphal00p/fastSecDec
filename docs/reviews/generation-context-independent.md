# Independent review of generation context adoption

Reviewed the caller-owned `GenerationContext`, additive `GenerationEvent` and
`GeometryReuseStatus`, the complete `generation/mod.rs` diff, the borrowed-map
adapter and all five new public integration tests. No actionable correctness,
ownership or native-reuse defect was found. This extends the separately
reviewed [exact geometry cache](geometry-cache-independent.md); it adds no
algebra, decomposition algorithm, expression cache, worker or global lifetime.

Each call still performs domain assessment before extracting supports or
consulting the geometry cache. Threshold, branch and upper-boundary failures
cannot be overridden by an earlier successful request with the same support.
Caller assertions remain part of the current result's metadata. Source
coefficients, prefactors, regular polynomial factors, coordinate symbols,
mapped residual checks, symmetry decisions, subtraction and Laurent expansion
are rebuilt from the current input. Only complete integer geometry is shared.

The original free `generate` entry forwards the same progress events through
the cache-free path. Its maps retain their previous owned movement; the context
borrows shared maps and creates owned chart/representative metadata where the
generated result requires it. Positional factor valuations, chart order and
original source IDs therefore keep their previous interpretation. The new
serializable status explicitly counts native maps before symmetry, exact
extraction or numerical kernel allocation.

Native completion cancellation still returns the native geometry error and
prevents insertion. Cancellation on a hit leaves its good entry; cancellation
at the later reuse event may retain newly completed geometry but never returns
a partial generated integral. These boundaries are stated at the API. Empty
integrals skip geometry. Capacity remains an entry bound, not a memory bound.

The five tests compare cold, warm and free generation at compact native roots,
alias maps, every retained chart and final kernel identity. An independent
analytic Laurent vector checks the compiled values. Changed coefficients and
both regular/prefactor weights have a distinct expected residue and finite
value; admission failures, assertion scope, domain changes and cancellation are
also exercised. These checks have scientific value beyond mirroring lookup
logic. No additional redundant cache test is requested.

The initial author five-test execution had three passes and two incorrect
test-placement expectations: existing generation extracts only wholly constant
vectors, so a constant pole alongside coordinate-dependent higher coefficients
belongs to the sector vector, not the exact offset. The author corrected only
those assertions to compare the same complete analytic vector with zero exact
offsets, and added an explicit changed-weight warm-cache control. The failed
log remains at `output/generation-context-tests.log`. No production correction
was needed. The corrected target passed **all five tests** in its exclusive
native runtime slot; reviewed evidence is
`output/generation-context-corrected-tests.log`. This review did not duplicate
the scientific run while the coordinated full workspace gate was active.
The subsequent serial workspace log
`output/geometry-context-workspace-tests.log` contains305 passed, zero failed
and18 explicitly ignored tests across57 suite summaries, including this target.
No benchmark or end-to-end performance claim follows from this review.
