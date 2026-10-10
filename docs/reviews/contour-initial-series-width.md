# Initial native Series request width

Reviewed 2026-10-10 independently of the implementation. The new
`CoefficientExpansionOptions.initial_relative_width` is a caller-selected
**starting request**, defaulting to one. It changes neither Laurent acceptance
nor the symbolic endpoint algorithm. No production performance improvement is
claimed without a matched physical run.

## Native API and mathematical boundary

The owner already exposes `SeriesDepth::relative` and `Series::absolute_order`;
source was checked at Symbolica `74225696`, `src/poly/series.rs`. FastSecDec's
existing composer calls `Atom::series(regulator, 0, SeriesDepth::relative(width))`
and performs the same native symbolic IBP/Taylor operations afterward. The new
option only replaces the initial literal one passed to that established route.
It adds no CAS, AD, valuation estimate or coefficient parser.

Every completed attempt still requires the native absolute remainder order to
be **strictly greater** than the requested maximum Laurent order. If insufficient,
the unchanged checked integer update adds at least one and otherwise the native
coverage deficit. Each retry rebuilds its request/body caches. A wider first
request is not a claim that a source has a particular valuation, and cancellation
or a truncated zero cannot certify a missing tail. The existing physical fallback
for unregulated endpoints runs before this option and retains its own admission
and error precedence.

The regular named-series path rejects zero/negative starts and starts above a
configured width cap before constructing requests or performing Series work.
Attempt, width and unique-request caps remain effective. The option cannot
suppress the absolute-coverage guard or change a failure into an accepted result.
An unnecessarily wide request may use more time/memory; the library chooses no
automatic workload-specific start.

## Public configuration, staging and cache ownership

The native option is reused directly by CLI TOML under
`generation.coefficient_expansion`. Missing and explicit-one values agree.
`skip_serializing_if` preserves the historical default JSON bytes; no staged
record or artifact schema is added. Existing staged options transport the native
structure unchanged. No Python surface expansion is part of this slice.

The numerical-dual formal-recipe key includes the start because an attempt cap
can make the same formula construction succeed or fail depending on this policy.
Both native key equality/order and the portable lookup digest distinguish opted-in
values. For start one, the exact historical flat lookup tuple remains unchanged;
for another start, the native serde payload wraps that tuple with the new value.
This is bounded caller-owned formula caching, not a new global cache or symbolic
identity algorithm. Symbolic endpoints and contour-only Jacobian differentiation
remain separate; this option does not reorder deformation and subtraction.

## Executable evidence

The maintained tests exercise existing public/native paths:

- Eight coefficient-composer tests pass, including two new controls: starts
  one/two/three produce exact equal coefficients with observed attempts `[1,2]`,
  `[2]`, `[3]`; requesting higher order from start two still retries `[2,4]`.
  Both Taylor and IBP compare against native physical-expression expansion.
  Zero, negative and capped-out starts reject before Series/request work.
- Three formula-key tests pass, including native and portable separation of the
  new policy alongside existing endpoint powers, prefactors and resource caps.
- The staged symmetry/multiplicity test passes with start two, checks the first
  observed request width after restoration, and preserves complete native-vector
  parity and source coverage.
- Eight public native coefficient-expansion tests pass. The two public
  coefficient-expansion process tests also pass, retaining actual
  progress/error reporting and failure without publication.

The standalone two-test new-control filter overlaps the eight composer tests.
The CLI native-default/TOML test also passes, including exact historical default
JSON bytes and explicit-one equivalence. These are **23 distinct focused tests**,
not an additional full-workspace count. Strict workspace/all-target Clippy passes
in 23.61 seconds; formatting passes. All owning gate processes closed and the
production freeze has no drift. Evidence and log hashes are in
`target/contour-initial-width-result.json` and
`target/contour-initial-width-source-freeze.json`; raw outputs remain ignored.
This review found no production-source blocker or missing native-owner operation.
The independent immutable optimized build and subsequent physical measurements
are separate acceptance steps.
