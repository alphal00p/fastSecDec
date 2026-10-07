# Native numerical-dual cache review

Scope: the generation-owned cache of original source evaluators and their
native dualized programs. This does not introduce a worker pool, external cache
I/O, evaluator prewarming phase, new algebra, or shared maximum derivative shape.

## Owner APIs and mathematical requirements

The pinned Symbolica revision `1deccb8538ccb91dc2c1e58fc0a2e900d2276bf4`
provides `Atom::evaluator`, `ExpressionEvaluator::vectorize`, `Dualizer` and
Numerica's `HyperDual`. Its `HyperDual::from_values` checks that every component's
ancestors are present (`lib/numerica/src/domains/dual.rs`, around line 1347).
`Dualizer::new` accepts exact structural-zero assumptions; these specialize the
resulting scalar program and are part of its scientific identity.

For one requested normalized coefficient with multiindex `nu`, the entire box
`0 <= alpha <= nu` is its smallest ancestor-closed shape. The existing per-request
construction already produces that box, including the epsilon axis. At a zero
face, the original polynomial must supply coefficients shifted by the known
monomial valuation, so the source shape's maximum is `nu + valuation`. Its box
is also the minimal native ancestor closure. Neither shape is enlarged to match
another request or sector. Structural-zero components remain in the shape while
native vectorization prunes their arithmetic; they cannot simply be deleted in
violation of the owner's ancestor requirement.

Earlier focused native evidence is retained under ignored
`output/deferred-sector-map-study/`: original `P=x^2+x*y+x^3` evaluated on map
jets `x=t, y=t*u` produces the exact shifted residual `1+u+t`, with normalized
Taylor coefficients, in eager, SymJIT O2, DoubleFloat106 and Float3322 routes.
The same probe demonstrates that inverting the unshifted vanishing jet yields
nonfinite values. This change preserves coefficient shifting before inversion.

## Shared ownership and concurrency

The source key remains the exact polynomial Atom, ordered native input Symbols
and complete compilation settings. A jet key additionally contains its ordered
shape and exact `(input, component)` zero mask. Different zero masks, input
orders, or settings never reuse the specialized program.

Previously one cache mutex remained held throughout source compilation and jet
vectorization. The cache now uses brief source/jet index locks and per-key
`std::sync::OnceLock` cells. Native construction happens after index guards are
dropped. Different keys can progress on the caller's existing workers; identical
keys initialize once and return the same immutable `Arc` owner. No global cache
or execution service is added. Each numeric evaluator still derives from a
clone of the exact native program.

Construction errors are retained as the same `KernelError::Compilation` message
for an exact key. Native initialization panics propagate to the existing caller
job boundary; standard `OnceLock` remains uninitialized and unpoisoned so a later
attempt is possible. Cancellation remains owned by existing native-unit/caller
boundaries; this cache neither converts cancellation into zero nor promises
interruptibility inside a native build. Completed cache values are not affected
by interrupted caller sessions.

Native optimization remains one-core and deterministic. Index insertion order
does not enter program construction, request assembly or artifact identity. The
existing serial, retained-session and reversed-dispatch artifact controls remain
the broader pipeline oracle.

## Validation

Focused new controls cover concurrent identical-key sharing and serial exact-IR
identity; zero-mask/input-order/settings separation; exact normalized valuation
shifts; unrelated source and jet keys finishing while another initializer is
held; shared errors and successful independent keys; panic retry; and minimal
ancestor shapes accepted by native HyperDual. Synchronization controls use
bounded channels, not a wall-time speedup threshold.

Source formatting and diff whitespace checks pass. All eight focused controls
passed with the workspace feature union on 2026-10-07 (0.01 seconds test time;
14.08 seconds incremental build), using:

```sh
env -u FASTSECDEC_SYMBOLICA_SOURCE_ROOT \
  CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=/usr/bin/clang \
  cargo test --workspace --lib generation::numerical_dual::native:: \
  --locked --jobs 4 -- --test-threads=1
```

The preserved log is `output/numerical-dual-cache-study/native-cache-tests.log`.
An independent source reviewer accepted the short lock lifetimes, native-owner
reuse, key separation and error/panic boundaries before this execution. The
coordinated milestone gates subsequently passed 544 native workspace tests
(26 explicit diagnostics ignored), 72 portable host tests, strict all-target
workspace Clippy and isolated bindings/stub Clippy. Performance is measured
separately in the formula benchmark review; no speedup follows merely from cache
correctness.

## Independent formula-pipeline source review

The separate formula pipeline was reviewed against the native cache boundary.
Exact ordered endpoint powers (including rational epsilon slopes), complete
prefactors, Laurent order, strategy and resource limits form generation-scoped
keys. The regulator, target coordinates and reserved source symbols belong to
one shared context. Full chart term ordering is preserved. Source polynomials
and maps remain with each chart and are supplied through that chart's request
evaluator; sharing the immutable `Arc<Recipe>` does not share numeric values or
native map programs. Exact unregulated/signed/epsilon fallback admission and the
zero-dimensional exact offset path remain separate.

Sorted native keys assign deterministic formula IDs. Existing opaque job owner,
stage, full-coverage and index checks admit reordered completion vectors before
the recipe IDs are used. Cooperative sessions retain completed discoveries and
recipe owners; their existing observer wrapper finishes the current native unit
before honoring a pause. The new pipeline does not widen jet shapes, merge
scientific point populations, or introduce a numerical zero proof.

Discovery, unique-formula preparation and chart assembly have separate enclosing
wall timers; nested symbolic fallback timers are suppressed to avoid double
counting. Review identified the one-time shared context setup outside those
timers. Its owner added `Context::prepare`, charging eligibility/symbol-reservation
work to Mapping in both direct generation and retained sessions. No remaining
source blocker was found. Integrated formula-cache scientific, cancellation and
phase controls remain the pipeline owner's separate acceptance gates.
