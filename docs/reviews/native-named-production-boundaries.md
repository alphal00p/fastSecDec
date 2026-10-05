# Prospective production boundary for native named coefficients

This is a source/interface review, not an adoption decision. The candidate now
produces the captured representative and builds native IR/O2, but its independent
original-expression oracles and cold complete-order numerical checks remain
open. The original production route stays unchanged until those gates pass.

## Place the change before the physical Taylor expression

`generation/mod.rs` currently passes each representative's mapped terms to
`subtraction::subtract`, then passes the returned physical Atom to
`laurent::expand`. Selecting names only in the latter step would retain the
observed large-expression construction. The useful private entry instead
consumes the existing mapped terms and returns native `AliasedAtom`
coefficients plus conditioning and progress facts. It must rejoin the current
pipeline before multiplicity, exact-only classification, global order padding,
chart-to-kernel association and `GeneratedSector` construction. Those owners
already inspect native root/image dependence and feed one native evaluator to
O2, tracking, MPFR, worker cloning and version-three persistence.

No new public coefficient container, instruction representation, function-body
serialization or evaluator path is needed. Keep `aliased_coefficients()` and
explicit lazy `coefficients()` materialization, and keep compile/save free of
implicit restoration. Do not copy the test-only `Captured*` or JSON protocols
into public generation APIs.

## Small private module split

Four responsibilities are sufficient; a general symbolic task framework is not
needed:

1. A shared endpoint-admission helper owns the existing rational affine
   exponent check, native floor/count conversion and subtraction-degree limits.
   The physical and named routes use this one policy. Retain the physical route
   as the control and exact unregulated fallback rather than copying its
   admission policy into a second implementation.
2. A native Series composition module owns the existing Taylor/IBP scheduling,
   independent scalar multipliers, grouping and native absolute-remainder retry
   controller. Symbolica owns Series multiplication, differentiation,
   factorial/division arithmetic and bounds. Formal coefficient names must stay
   linear: no inverse, power, logarithm or scalar multiplier may depend on them.
3. A local coefficient-request module owns source-body deduplication, native
   DER decoding, literal substitution, the separate unsubstituted/full-request
   caches and native alias lowering. The restricted interleaved resolver is a
   small helper inside this owner. All body/cache memory dies with the current
   representative/attempt; no global callback captures are introduced.
4. A small coordinator returns the complete ordered native coefficients and
   explicit generation/conditioning observations to the existing assembly.
   It selects a supported representation without reconstructing geometry,
   prefactors, numerator weights or chart identities.

Moving the current long test-only file verbatim into production would not yet
meet these boundaries: its hardcoded retry limits, missing observer polling and
prototype-only statistics/JSON wrappers need deliberate integration first.

## Admission and domains remain the existing owners' responsibility

`ParametricIntegrand::new` already checks parameter-independent prefactors and
exponents, polynomial numerator roles, and projective homogeneity.
`mapping::map_terms` performs exact valuations and calls
`domain::check_residual` on singular factors before producing regular bodies.
That residual check proves a nonzero constant and the required zero/one faces,
including their intersections. Preserve these checks on every call and route;
an `assume_no_threshold` assertion does not establish unresolved boundary
regularity. Upper-boundary affine-chart limitations remain unchanged.

Interleaving applies only to independent native coordinates or exact zero/one
faces of admitted regular coefficient bodies. It neither evaluates singular
limits nor justifies new branch choices. Composed/swapped arguments and other
constants keep the original simultaneous route. Regulator-dependent polynomial
numerators, negative integer residual powers, complex polynomial coefficients,
signed orthant infinity maps and zero-dimensional/projective maps still need
their existing full admission and complete-vector tests.

Retain the exact physical fallback whenever a divergent endpoint has zero
regulator slope. A truncated coefficient series cannot prove that an exact
boundary derivative vanishes to all orders; removing this fallback would change
the existing `UnregulatedEndpoint` and polynomial-pruning contract. Taylor and
IBP require separate coverage. The successful actual candidate is Taylor only;
small IBP controls do not establish large-input IBP completion.

## Conditioning facts must describe the new representation honestly

The physical route records cancellation rows from retained Taylor remainders.
The candidate conservatively keeps terms whose native truncated coefficients
may vanish, so its piece count and conditioning rows need not equal physical
counts: the actual proof has 1,536 formal pieces versus 872 physical pieces.
Production must derive its own conservative rows from the named endpoint
schedule or from all mapped endpoint powers. It must never rerun physical
subtraction merely to obtain these rows.

A componentwise maximum derived from every relevant endpoint count dominates
all such rows on the open unit cube, where `-log2(x)` is nonnegative. Checked
sums and dimensions must remain in the existing `Cancellation` owner. This is
conservative input to the existing precision heuristic, not a certified bound
on numerical error. It may increase checks and rescue work. A bound row must
not be described as an actually retained physical remainder. Before adopting
it, settle the public accessor/documentation meaning of `cancellation_terms`
and retain an explicit representation/bound qualification in generation
observations. No new estimator, sampled cancellation test or silent reduction
of precision policy is warranted.

## Resource, cancellation and status decisions still required

The prototype's twelve attempts and width cap 128 are explicit experimental
limits. Silently making them production limits could reject previously
supported highest orders. Choose a documented resource policy or an honest
existing-route fallback; never accept a coefficient vector until the actual
native remainder exceeds the requested signed maximum. Known literal zero and
finite native-series remainder behavior require their separate proven paths.

Enforce the existing piece/degree limits at each growth boundary, including IBP
boundary pushes, using checked counts. Add caller cancellation checks between
native series calls, endpoint axes/pieces, request-lowering operations and final
assembly. A single native CAS call remains noninterruptible unless its native
API supports cancellation; do not claim otherwise. Errors and cancellation
must not emit a successful `Complete` or a usable partial integral.

The combined composition does epsilon work before/during endpoint operations.
Existing Subtraction/Laurent phase timings therefore cannot simply be populated
with copied or overlapping durations. Keep existing public observers working,
and make any additive combined-stage/representation observation explicit before
CLI use. Retained formal piece counts must be labelled as such. New symbolic
representation may change native program/content identities; preserve old
artifact readers and checkpoint identity rules rather than forcing different
native bytes to share an identity.

## Required acceptance before default production use

First complete the three independent original-expression oracles and the cold
reader over the entire signed-order union, including the candidate's additional
leading order. Then run the ordinary public generate/compile/cold-load path on
small analytic Taylor and IBP integrals, both negative and positive requested
maxima, Gamma and hidden complex bodies, exact-zero/constant-only cases,
unregulated/fractional/essential errors, domain failures, cancellation and limits.
Use existing weighted adaptive/forced-replay and worker controls at the same
rounded coordinates. Check metadata, multiplicity and no unresolved names at
the executable boundary. Finally establish whole-graph completion independently;
one representative and small controls do not close that capability gate.
