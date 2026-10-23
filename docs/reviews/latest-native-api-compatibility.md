# Latest native API compatibility

2026-10-23 — `fix/latest-native-api-compatibility`; focused validation complete.

The current manifests and locks select Symbolica fork
`87fd1b36d991ee8151b39c2916109dd4ef5105df`, combining the prepared API owner
`a5f94e6550d78b3c5fd4ce9b1d442ad6daefd299` with reviewed Numerica error-range
and native HPL additions, based on upstream
`170ba932d0cf73508da34cc78fbe82361158147e`. The prepared-root and shared
direct-vector wrappers are supplied separately by
[Symbolica PR #127](https://github.com/symbolica-dev/symbolica/pull/127).
Numerica and Graphica share that Symbolica owner. FeynKit resolves to
`fb6746c520ace038b6368b06d3122940cec72c88`; SymJIT resolves to registry
`2.28.1`. See [the dependency table](../DEVELOPMENT.md) for the full selection.

Root, portable-kernel and Python lockfiles agree on these native identities and
pass `cargo metadata --locked` with single native owners. The combined owner
contains [Numerica PR #128](https://github.com/symbolica-dev/symbolica/pull/128)
and [HPL PR #95](https://github.com/symbolica-dev/symbolica/pull/95); FastSecDec
does not duplicate either implementation. Focused consumer validation of this
combined selection passes within the scope recorded below.
Direct and transitive FeynKit consumers share the `branch=feynkit` Git query:
a separate `rev` query can duplicate native type owners at the same SHA. The
branch alone is not a revision pin. Targeted `cargo update --precise` selects
the tested feature-branch commit without introducing a second native owner;
commit the exact lockfiles together and validate with `--locked`. Historical
fork-fix claims are not evidence for this
source selection.

## Native caller migration

Composition now uses `EvaluatorComposer::new_with_inputs`, `add` and
`ComposerSlot`. Raw `Slot` remains for evaluator instruction inspection.
Composer handles stay local to their owner; shared caches retain evaluators.
Both second projection composers obtain fresh input handles before adding the
checked evaluator. Input/output order, normalized Taylor seeds, once-only
Jacobians, callback ownership and positive/interior guards remain explicit.

FeynKit supplies typed graph numerators and `SymbolicTensor` interfaces.
Numerator, projector, numerator prefactor and overall factor compose once;
measure weights retain their existing role. Native Lorentz-dimension mapping
preserves scalar coefficients, routing precedes simplification, and retained
free-index interfaces reject ranked zeros with open ports. Represented payload
conversion retains those interfaces and rebuilds local fragments once, keeping
the rounded aggregate as provenance. Graph/family adapters and fixtures retain
`D` dependence and physical mass-sign conventions. Focused native scientific
regressions cover these contracts; broader configurations remain outside the
validation scope below.

## First independent review and fixes

The first review found three defects and no production composer regression:

1. Python tensor subclasses lost their interfaces through expression inference.
   The adapter now copies `TensorExpression.structured()` before raw fallback,
   retaining ranked-zero numerator/projector interfaces.
2. Raw Python source indices differed from native graph identities. Raw inputs
   now use `NumeratorInterface::admit`, matching native DOT index admission.
3. Encoded index metadata could bypass represented-payload Float refusal. Both
   exact and `PrimitiveFragmentsV1` paths now inspect original native labels and
   canonical ports, including labels retained only by ranked zeros. The bounded
   scanner borrows `UserData::Atom`, uses `AtomTreeIterator` and native
   `CookSettings::indices`/`ReversibleEncoding` recognition per symbol. Shared
   byte/node/depth/rational-bit limits apply; labels are neither exactified nor
   renamed, and the original cooked tensors remain intact.

Regressions were added for Python structured/raw admission, encoded Float and
exact labels, retained zeros and resource limits. The native input and encoded
label regressions pass; the two focused Python input tests also pass with the
upstream caller fix described below. Source review records limited permutation/guard-isolation
coverage in projection fixtures and no dedicated nested/scoped encoded-label regression;
arbitrary non-normalized custom metadata is not certified by this review.

## Validation and evidence

The final combined owner `87fd1b36d991ee8151b39c2916109dd4ef5105df`, with
FeynKit `fb6746c520ace038b6368b06d3122940cec72c88` and OneLOop
`723e6b19c4d19937f3b2f4c69c21f7a4e235175f`, passes the authorized focused checks:

- 33 exact-name native core tests, including composer ownership/order, runtime
  branches, the original affine uncertainty assertion, native tracking and
  shifted-input uncertainty controls.
- Both native Python admission tests, executed from the frozen binary with the
  glibc loader matching the installed Python library. This is not a full Python
  test-suite run.
- `cargo check --release --locked --offline --tests` for the standalone portable
  package, with the native feature absent. This is compile-only validation.
- The complete native CLI release build, including threshold decomposition.
  The final CLI was frozen without executing another scientific calculation.

The respective core, Python, portable and CLI compilation times were 691.980,
706.415, 72.809 and 450.751 seconds. All successful stage receipts retain exact
commands, source/lock/binary hashes, before/after metadata and verified supervisor
cleanup. The source/lock snapshot is identical across the completed stages. No
full-library, full Python-suite or workspace test pass is claimed.

The added affine callback regression uses an exactly represented native dyadic
center and weight near `1e-200` at 192 bits with valid decimal precision 10. It
checks preserved nonzero centers, refusal of unrepresentable propagated error
in both components, and exact-zero acceptance after refusal. It passes with the
native [Numerica fix in Symbolica PR #128](https://github.com/symbolica-dev/symbolica/pull/128).
Every original assertion remains unchanged. The earlier zero-center fixture
requested precision outside its constructor contract; the new weighted test
provides the valid-input regression. The native owner's isolated validation
separately passed 242 native and 28 focused portable tests.

Earlier scientific validation on the prepared-API owner passed 47 tests across
eight suites, including typed numerator contraction, signed mass terms, dimension
mapping, retained-zero refusal and encoded-label bounds. The gamma constant test
also passed there. Those historical results retain their original dependency
identities; they are not reported as new runs on the final combined owner.
Independent round-two and final consolidated source reviews found no concrete
blockers and confirmed all three original admission findings resolved.

The one-line default-preserving caller fix in
[FeynKit PR #215](https://github.com/alphal00p/gammaloop/pull/215) supplies the new
`horner_use_cpe: bool` argument in `spynso3`. The single PyO3 owner remains
`0.28.3`. The development-only OneLOop provider uses Symbolica's existing complex
polylog callback through [OneLOop PR #11](https://github.com/alphal00p/oneloopmaster/pull/11);
its 16 native tests pass independently. FastSecDec adds no special-function
implementation.

Infrastructure evidence remains separate from test results. An earlier core
attempt failed with filesystem `ENOSPC`; a later portable admission was refused
before Cargo by a conservative disk-decline rule. Both records are preserved.
After external capacity recovery, the final two builds used the revised absolute
disk policy. The final CLI's outer disk monitor was interrupted while its
supervised Cargo job continued; monitoring reattached to that same runner, and
the interval is explicitly recorded. The job was not restarted, its memory/time
supervisor remained active, and final cleanup and source checks passed.

Latest native constant normalization can conservatively produce complex layouts
for otherwise real gamma-series coefficients. Test assertions now map explicit
order/component pairs, retain all orders and strict real tolerances, and require
every imaginary component to vanish. No production reality inference changed.
Deep inspection of both zero-dimensional generation modes gives maximum real
residual `2.220446049250313e-16` against tolerance `2e-14`, with all four imaginary
constants exactly zero. This is constant evaluation, not calibration sampling.
The additional test changes passed an independent third source review and the
focused gamma test passes.

The enclosing pilot accepted the scoped P14 ordinary-Havana calibration through
the finite coefficient at three significant digits. It uses three predeclared
independent fixed allocations, complete-batch Laurent covariance, and an exact
all-chart higher-pole proof. The statistical intervals rely on the stated
proposal/CLT assumptions; the pole proof relies on the native decomposition
identity. Frozen sampling uses Distance-mode plain `f64`/double-double arithmetic
and excludes the affected tracked-uncertainty path; load-time EPF scratch
initialization remains explicitly qualified. This acceptance does not establish
whole-library migration readiness or complete the enclosing pilot's global flow
gate. No numerical coefficients are published by this API migration.

Evidence in the enclosing workspace's
`scratchpad/5l-pilot/fastsecdec/results/`:

- `independent-native-migration-review.txt`: first review and its three findings.
- `encoded-index-fix-evidence.txt`: native API reuse, bounds, regressions and limits.
- `source-lock-identities.json`: exact FeynKit identity and lockfile provenance.
- `final-public-feynkit-lock-identities.json`: tested FeynKit caller revision,
  targeted public lock update and single-owner metadata checks.
- `final-public-combined-lock-identities.json`: exact combined native owner,
  three targeted lock updates and locked metadata checks.
- `final-native-review-round2.txt`: independent assessment of the three fixes.
- `final87fd-consolidated-source-review.txt`: final source/pin review and coverage limits.
- `gamma-regression-review-round3.txt`: explicit component-layout test review.
- `final-check-summary.json`: preserved earlier compiler/runtime results.
- `post-repair-completed-summary.json`: final 33 core and two Python passes,
  portable check, CLI build, exact commands/hashes and infrastructure qualifications.
- `p14-calibration-acceptance.json`: hash-bound scoped calibration receipt,
  including statistical, higher-pole and frozen-dispatch evidence.
