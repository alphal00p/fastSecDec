# Native kernels for certified regular algebraic boundaries

The existing complete-vector compiler now accepts an issued regular algebraic
secant family through `KernelSet::compile_threshold_family` and the shared
detached compilation plan. This is a restricted certified family constructor,
not a general cell-to-integrand resolver or a new CLI preparation capability.

## Degree-independent arithmetic

Each boundary retains its full defining polynomial and selected branch.
Symbolica prepares the polynomial and derivative evaluators once, refines the
selected root with its native bracketed solver, and supplies symbolic implicit
derivatives. There is no radical formula, all-root search during sampling,
replacement AD, or finite-series approximation to the bulk integrand.

The initial numerical recipe, `StaticExactF64BracketV1`, requires a certified
uniform regular cylinder with constant bracket endpoints exactly representable
in finite `f64`. Arbitrary rational endpoints can remain valid geometry while
being unsupported by this recipe. This intentionally tightens admission from
the earlier untracked map probes. Moving brackets, collisions, nested towers
and the general all-face atlas still need their own certified constructions.

The issued family retains both actual cells, causal phases, full factor
associations, closed-face units and its common regulator strip. Subtraction
and Laurent construction remain symbolic. Root-bearing constants retain their
original unit-cube sampling contribution instead of being incorrectly folded
into a rational exact offset.

## Runtime, artifacts and numerical policy

Detached root helpers contain optimized native IR, branch association and
small numerical metadata. They retain no GCAD tree or source graph. Native
format v17 restores the registered callback and helper owners before mapping
the complete evaluator. Existing indexed records carry only their required
helpers. Loading does not repeat geometry, symbolic generation or Horner/CPE;
existing backend translation and fallback compilation remain possible.

Mathematical helper identity excludes optimizer choices; transport identity
includes the complete optimized bytes. A callback is associated with both
its local name and full branch/program tag, so merged records cannot replace
one another's branches. Historical rational formats and identities remain
supported. Artifact restoration is trusted execution, not geometric proof
replay. Ordinary native integration can restore these kernels without GCAD;
root-bearing v17 is currently unsupported on portable consumers.
The current issued family has one root; canonical metadata for a future
multi-root family with reused local names remains separate work.

Root kernels require the existing validated stability policy. Acceptance uses
the full native tracked vector, including weights, at both successive rescue
precisions. Root and contour callback failures remain independent and cannot
be hidden by later arithmetic. Ordinary kernels skip root callback bookkeeping.
Known exact controls are constructed exactly; incoming uncertainty is retained.

Numerica's native uncertainty representation is range-limited and is not a
certificate. Unrepresentable tracking refuses the point. A near-collapsed
width of order `1e-20` is recovered; the `1e-200` control explicitly refuses
instead of returning a false zero. No sample, pole component or covariance
entry is silently discarded, and no tolerance is loosened to accept it.

## Scientific control and remaining performance work

The complete test integrates `(y^5+y-x-i0)^(-1-epsilon)` over the unit square.
An independent native symbolic primitive and exact isolation of `r^5+r=1`
give the finite imaginary reference `pi*r = 2.371518130639618`.

Eager and SymJIT independently accept 16,384 points each, using Kuo 1024,
eight shifts per chart, Korobov3 and seed `202610114101`. Both return a finite
imaginary part `2.3715181309694566 ± 7.391691376253193e-9`. The complete
six-component Laurent vector and 36 covariance entries agree between
backends. The independent integrated oracle checks the pole cancellation and
finite imaginary part; it does not provide an analytic reference for every
coefficient.

These diagnostic runs take 48.250 s and 48.283 s, respectively. They rescue
11,504 points (70.21%), predominantly at 256 bits and once at 512 bits.
This establishes a correctness baseline and a substantial optimization need,
not production performance. The evidence uses the coherent public `ae0c82b`
Symbolica/Numerica dependency; old-owner results are superseded.

Native API/source/probe evidence and independent generation, resolver and
coordinator reviews cover the callback, metadata, precision and ownership
boundaries. Maintained tests cover degree-independent roots, branch isolation,
codec restoration, complete-vector sampling, failure handling and selective
loading. Weak-reference checks establish owner release, not aggregate RSS.
Local source and raw evidence remain under
`target/no-deformation-algebraic-kernels/`. Joined regression acceptance is
recorded with the consuming-workspace milestone.

The coordinator verified all 44 source preimages and 75 evidence digests before
importing the frozen slice. Its handoff SHA-256 is
`d7483399a2153684f8d52eb6c495c134090430c3bf50f08aed8fc246422e6540`.
The maintained tests, eight inherited regular-map controls, 20 existing
rational factory/serial/indexed controls and strict library/test lint pass.
Fresh restoration also passes in a separately built default native process
that has no threshold feature or symGCAD dependency.

The joined workspace run passed 689 native tests and retained 22 ignored
tests; its only failure was an inherited assertion expecting the old
contour-specific callback error prefix. After updating that assertion to the
shared native-callback diagnostic, all five affected evaluator tests passed.
All 88 portable tests and strict workspace/all-targets Clippy passed, as did
formatting and whitespace checks. This records the full run and focused
test-only correction separately; the full command was not rerun afterward.
