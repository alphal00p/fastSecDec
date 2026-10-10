# Native GCAD admission and dependency audit

This is an independent source and ecosystem-reuse review of the initial
`threshold::gcad` boundary and contour-capability cleanup on `no_deformation`.
Source review passes for **verified generic geometry only**. It does not accept
an endpoint resolver, threshold integration recipe, or numerical benchmark.

## Public dependency and native ownership

[symGCAD PR 1](https://github.com/alphal00p/symGCAD/pull/1), revision
`a1132d4f4545c7784b3ec61239a05485e544b403`, replaces the missing private Symbolica
path with a registry dependency. Standalone root patches retain the original
public Symbolica/Numerica/Graphica revision `75f8350`; only three source fields
change in its lockfile. No solver Rust code changed. The standalone library,
root, solver and FastSecDec-import controls passed **225 tests**, with one
existing ignore, without a bootstrap checkout. An actual downstream dependency
using FastSecDec's `74225696` backend passed seven native control groups,
including an explicit shared polynomial type assignment, independent proof
verification, tamper rejection, parameter fibers and algebraic root refinement.

The owner packaging change has independent source/provenance review. Its
repository-wide formatting check reports three unchanged upstream files; this
was retained and disclosed instead of expanding the packaging patch. Build and
test evidence is under `target/no-deformation-gates/symgcad-dependencies/`.
The [dependency milestone audit](no-deformation-dependencies.md) records the
public packaging and subsequent native series repair separately.

The current FastSecDec metadata snapshot contains exactly one Symbolica and
Numerica `74225696` identity and the public symGCAD revision above. The optional
`threshold-decomposition` feature implies `native`. Separate portable and Python
metadata snapshots exclude symGCAD; portable retains its existing
Malachite/Astro backend. This is dependency-graph evidence, not a new WASM or
Python threshold implementation claim.

## Mathematical association and verification

- The adapter reuses `ParametricIntegrand`, native `Atom`, exact Numerica
  rationals, Symbolica's rational polynomial conversion, and symGCAD's own
  formatter/parser, solver and verifier. It introduces no graph parser,
  polynomial algebra, root isolator, or independent sign engine.
- Ordered aliases retain their original Symbolica symbols and distinguish real
  runtime parameters from integration coordinates. Native polynomial axes are
  checked before formatting. Exact finite IEEE bindings use native rational
  conversion; undeclared coefficients, complex geometry and nonpolynomial
  geometry are refused. Complex numerator/prefactor data remains in the density.
- Sign splits share only exactly equal specialized polynomials. Negative
  multiples remain distinct; every original term/factor retains its split
  association, unmodified exponent and branch semantics. A declared positive
  factor is checked after verification on every retained cell, rather than added
  as a domain constraint that could silently discard a negative region.
- Request equality includes the complete native density, declared prepared
  domain, binding map, ordered aliases, factor associations and native problem
  settings. Thus equal zero-set geometry is insufficient to transfer a proof
  between different densities or declared preparations. Derived equality on the
  three existing parametric types changes no arithmetic.
- Parameter groups precede integration groups, and verified output must preserve
  that role order as a complete permutation. Raw cell root selectors and their
  index domains remain owned by the native result; a cell view borrows that owner.
- Verification requires matching request evidence, `CompleteGeneric`, and a
  successful independent native verifier. Incomplete or foreign evidence cannot
  construct the verified type. Raw cells are retained rather than replacing the
  proof by a count or by merged cells alone.

The reviewed native engines either preserve the requested domain or replay their
explicit restriction to an original input inequality. Their special sign-filter
representations are not permission to discard a physical sign region. Actual
causal phases, Jacobians and epsilon expansions are not evaluated by this layer.

## Execution and remaining boundaries

`solve` is synchronous, requires one native worker in its request, and rejects
implicit ordering-search orchestration. The caller still owns scheduling and
hard wall/RSS enforcement: a cooperative library checkpoint cannot interrupt
every CAS operation. Native problem validation currently allows at most
65,536 MiB and 7,200 seconds in its declared limits; those fields are not a hard
RSS guard for a direct call. Native verification also remains caller-bounded.
Durable workers must call `solve()`, retain its raw evidence, and then call
`verify()`. The `solve_verified()` convenience consumes evidence on a failed
verification and therefore is not itself a durable failure-receipt workflow.

Prepared geometry is explicitly a declaration, not a proof that it covers an
original projective integral. Projective input is rejected until a proved
measure-preserving preparation exists. `CompleteGeneric` leaves exceptional
loci and all closed-face, branch, endpoint-regularity and analytic-continuation
obligations to later stages. Monomial powers and auxiliary-regulator atoms are
retained, but no multiregulator family, auxiliary-pole cancellation, physical
source lineage, artifact codec or executable threshold recipe is implemented
here. Native atom equality currently belongs to the matching native state;
there is no fresh-state serialization claim for request identity.

One later measurement opportunity is repeated conversion of equal F/U atoms
across input terms before polynomial deduplication. Reusing native conversion
results within one request may help; this review makes no unmeasured speedup
claim and requests no alternative algebra implementation.

## Contour capability regression scope

The seven-file cleanup changes contour admission from “anything other than
undeformed” to `ProgramRecipe::is_contour()`, explicitly matching the three
existing contour recipes. Its current truth table is unchanged. Mapping,
shared preparation, Jacobian-plan admission, empty-owner descriptors and runtime
capability use the same predicate. No recipe IDs, saved formats or source-sector
identities change. A future threshold recipe will still require its own positive
admission path and versioned one-to-many lineage; this predicate alone does not
implement those facilities.

The reviewed contour/parametric executable controls passed 29 tests, and the
streamed generation controls passed 16 tests. The integrated native adapter
gate passed **8/8 tests in 0.04 seconds**, with public symGCAD `a1132d4` and the
single Symbolica/Numerica `74225696` graph. This covers original signed factors,
complete exponents, shared geometry, exact rational specialization, parameter
ordering, proof tampering, foreign identity and unsupported input. The initial
dependency build took 11m55s; it is not solver runtime. The executable receipt is
`target/no-deformation-gcad-adapter-tests.log`. Raw metadata and capability logs
remain ignored under `target/no-deformation-gates/`.
