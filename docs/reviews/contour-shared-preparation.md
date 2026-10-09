# Shared native preparation for contour recipes

2026-10-09. Additive native preparation APIs reuse one geometry and one complete
monomial-extracted chart across selected recipes. This is preparation plumbing;
dynamic production and the complete Phase B acceptance remain separate gates.

## Public caller-owned units

- `prepare_recipes_with_runtime` validates the distinct requested recipes,
  computes the union of required supports once, and returns a
  `PreparedRecipeSet`. Its recipe contexts reference the same geometry records
  and canonical physical source identity.
- `prepare_chart_source` imports one source chart, performs the existing native
  substitution and monomial extraction, writes an immutable
  `PreparedChartSource`, and releases the heavy expressions.
- `discover_prepared` imports that chart and applies one recipe before entering
  the existing exact symmetry, formula and coefficient pipeline.

These calls construct no threads, processes or numerical execution loop. The
CLI can recycle a process after each shared-source or recipe-specific job.
Coordinator objects contain record references and compact identities, with no
native residual expressions or completed evaluators. Existing single-recipe
preparation/discovery entry points remain supported; ordinary caller-retained
generation semantics are unchanged.

## Native reuse and semantic preservation

The original mapper's Symbolica substitution, factored residual extraction and
sparse fallback are now shared internal functions. No polynomial parser,
valuation algebra, determinant, root solver or serializer was introduced.
The source record uses the existing native Atom/context codec.

Residual records retain the complete chart dimension, native map, ordered target
symbols, endpoint powers, prefactors and factorwise F/U semantics **before**
combining terms or restricting faces. Zero-exponent causal and positive factors
remain present whenever a selected recipe needs them. The sorted first recipe
may be undeformed; that does not determine which declarations survive.

Numerical-dual undeformed preparation keeps its existing opaque source mapping
and lazy evaluator programs. A mixed recipe record can retain that compact
source representation alongside the complete residuals needed by deformation.
It does not replace the undeformed path with a dense expanded numerator.

Selected deformation and term combination occur only after loading the shared
source. Exact symmetry is proved on that selected complete density. Formula
lookup hashes can coincide across recipes because native subtraction templates
omit their regular bodies; admission still requires the recipe and execution
source digest, followed by native key equality. A physical source identity is
never substituted for that execution-context fence.

## Identity and record checks

The prepared-source receipt binds physical source identity, chart index, map
record and original dimension. Restoration checks the immutable record digest,
stored receipt identity, native map equality and ordered target symbols against
the selected source context. Contour discovery refuses a prepared source that
did not retain the branch declarations. New prepared-source records have their
own explicit record kind; existing selected chart/formula payloads are reused.

## Checks and current limits

The initial live core check passed. Four focused shared-source tests passed
(0.29 seconds), covering one geometry construction and one native extraction,
zero-exponent declarations, cancelling source terms, opaque undeformed dual
mapping, foreign receipt rejection and complete Laurent-vector pointwise
agreement with selected generation in both generation modes and Taylor/IBP
subtraction. The local log is `target/contour-shared-preparation-tests.log`.
The extraction count uses the existing test-only mapper instrumentation; it is
not a memory measurement. No RSS or wall-time improvement is claimed from source
inspection alone. Independent contour science and the wider milestone gates
remain coordinated separately.

The companion envelope API admits independently enclosed norm/bound primitives
into the **same** native arithmetic coefficient assembly. Existing methods use
the original physical expressions; native alias roundtrip controls verify that
checker primitive slots reconstruct them without a separate convolution or CAS.
Persisted certified checker programs and runtime admission remain the runtime
owner's next slice.


## Independent source and durable recovery review

The foundation reviewer inspected shared preparation, native restoration and
the additive CLI worker/journal requests independently. No blocking defect was
found. The four scientific controls above exercise the important ownership
limits: an undeformed first recipe cannot discard zero-exponent F/U declarations;
cancelling terms retain declarations until recipe application; numerical-dual
undeformed sources preserve their opaque path; foreign source/index/map and
false declaration flags are rejected. Complete Laurent outputs agree with
selected generation in both modes and both subtraction strategies.

Each recipe applies its own complete deformation before native symmetry and
formula discovery. Formula-key equality alone cannot authorize cross-recipe
reuse: the native and CLI receipts also require the exact selected recipe and
execution-source digest. Shared prepared-chart receipts bind physical identity,
map record and dimension; the native consumer additionally compares restored
map content and ordered target symbols. Heavy payloads remain immutable native
records consumed by individual jobs, rather than accumulated coordinator Atoms.
This is an ownership inspection, not a measured aggregate-RSS claim.

CLI review covered `PreparePrograms`, `PrepareChartSource`, `DiscoverPrepared`
and the common journal admission helper. Issued intent is persisted before
work, accepted receipts are hashed, and referenced heavy records are verified
before reuse. The new runtime recovery test simulates a coordinator dying after
a durable child receipt, changed worker count, foreign recipe/source/map data,
cross-recipe formula responses and a truncated source record. The focused CLI
recovery gate passed all seven tests. The complete CLI package passed 157 tests
with seven ignored across 24 executables. The complete core library passed 292
tests with 16 ignored, including the final requested-map receipt check. The
portable and strict workspace Clippy gates remain in progress at this review
point.

The next orchestration increment must persist the canonical requested recipe
family before taking the completed-resume shortcut, use separate recipe job
namespaces and share the final publication path. The current changes are
additive worker primitives, not yet that public family coordinator. Dynamic
production admission remains separately closed pending complete certified
checker and factor-identity validation.


The final full portable consumer passes 73 tests with no failures or ignored
tests. Strict workspace/all-target Clippy and formatting pass. The CLI gate
precedes the final requested-map-only admission optimization; the full 292-test
native gate includes that change and its foreign requested-map regression.
The public family coordinator and production dynamic recipe remain pending.
