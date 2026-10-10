# Selected dynamic recipe generation and ownership

Historical foundation record, 2026-10-09, followed by dated acceptance updates
below. That initial slice prepared one selected mathematical program without
enabling dynamic production. Both dynamic constructions now pass the public
analytic gate, including empty/zero-dimensional inputs; see the final sections
and [the current milestone audit](contour-dynamic-milestone-audit.md). Physical
dynamic execution and complete Phase B acceptance remain pending.

## Native boundary and reuse

`GenerationOptions.program_recipe` uses the existing native `ProgramRecipe`;
there is no second recipe enum or independent boolean source of truth. The
default remains undeformed. Public CLI/TOML/Python contour capability steering
continues to select the fixed recipe until complete family orchestration is
ready. Fixed generated kernels retain their v9 format.

The polynomial recipe calls the previously audited native `DynamicEnvelope`,
Symbolica-built `RootProgram`, implicit derivative hooks and `SmoothContourMap`.
The complete local-strength map is differentiated before Taylor/IBP subtraction
in both generation modes. No root solver, derivative engine, polynomial codec
or determinant implementation was added. Factored residuals remain native Atoms.
The complete chart's positive factors are discovered before its envelope;
subtraction faces restrict the same map and structural counts.

At this initial milestone the sign-aware selector reported an explicit
unsupported error because its cancellation-resistant positive-part runtime
callback was still pending. An empty dynamic input also reported an explicit admission error pending its
empty-recipe handling. Neither case silently selects another recipe. The native
kernel runtime then rejected dynamic production pending its complete
certified checker and scientific gates.

## Ownership and staging

Mapped, prepared and expanded chart jobs retain strong native helper owners.
Representative assembly transfers them into generated sectors and integrals;
owned compilation jobs retain them independently. Zero-dimensional exact charts
keep their descriptor when stochastic sector storage is omitted. Raw checker
sources are compact ordered native Atom outputs and a structural schema; the
whole `DynamicEnvelope` is not retained twice. Optimization and persistence of
the final dynamic checker remain a later runtime slice.

The staging envelope is explicitly version 2. Its descriptor/helper bytes are
restored before native Symbolica state and Atom import, which matters in a fresh
process with no live callback registry entries. Chart payloads also retain the
native context-aware checking-source DTOs. Version 1 staging is rejected rather
than interpreting its old contour boolean as an enum or resuming mixed work.
This staging change does not alter finished fixed v9 artifacts.

Preparation, chart, formula, symmetry and sector receipts carry the selected
recipe. Symmetry receipts also retain the execution source identity, and native
admission rejects mismatched receipts even when numerical chart indices match.
Exact symmetry is proved on the complete selected deformed density. Hashes
continue to locate candidates only.

The public geometry-free `generation::source_identity` helper streams a typed
physical-input schema using Symbolica's native canonical Atom strings into the
digest. It retains domain, ordered coordinates/runtime symbols, regulator,
factor roles/semantics and named mass constraints. It excludes generation
options, recipe selection, target coordinates and the exact reserved deformation
runtime symbols. Arbitrary physical symbols sharing the contour namespace are
retained. Streaming preparation calls this same helper.

Final source review caught and corrected an earlier transport-digest assumption:
Symbolica's context-aware native serialization intentionally contains local
symbol IDs and registered polynomial coefficient rings. It is appropriate for
restoration and immutable staging receipts, but not canonical physical identity.
The owner API/source tests for `AtomCore::to_canonical_string` already establish
name/attribute-aware canonical ordering; no separate algebra canonicalizer was
added. A fresh-worker regression scrambles both symbol and coefficient-ring
registration, compares physical identities and verifies that the old state
transport digest actually changes. All three identity tests pass, including
direct admission of the returned digest to `ProgramArchiveWriter`; the log is
`target/contour-source-identity-tests.log`.
The recipe-specific source record still retains the complete execution context;
its transport digest fences that staged work, while physical source identity
can be shared across schedules and recipes.

`ProgramData::select` applies only to prepared charts before exact coefficients
exist. Completed payload projection must use the native descriptor operation
with actual exact expressions so their referenced helpers are retained.

## Validation status and remaining work

Focused controls cover the analytic subtraction integral
`x^(-1-eps)/(1/4-x-i0)`, whose pole is `-4/eps` and finite part is
`-4 log(3)+4 i pi`, in symbolic/numerical-dual and Taylor/IBP combinations.
Separate staging controls compare physical identities across recipes, reject
foreign recipe receipts and old staging headers, and restore/compile/evaluate
the selected dynamic source in a fresh worker process. The focused generation
filter passed 2/2 tests and the streaming filter passed 12/12 tests, including
the fresh worker's numerical callback evaluation after dropping both its
generated object and compiled kernel. Logs are retained locally as
`target/contour-generation-recipe-tests.log` and
`target/contour-streaming-recipe-tests.log`. These controls test native source
generation and helper ownership; they do not bypass or change the production
runtime admission gate. The full-core and CLI milestone gates remain pending.

The next native slice shares geometry and heavy residual extraction through an
immutable `PreparedRecipeSet` and `PreparedChartSource` spool. It must retain the
original full chart dimension and all declared positive factors before any jet
mask or face restriction. Each active worker then consumes one chart/recipe,
persists its sector and releases it. Caller-retained ordinary generation
semantics remain explicit. Complete multi-recipe CLI orchestration, optimized
dynamic checking programs, sign-aware production, certified runtime admission,
and fixed/dynamic variance measurements remain pending.

## Empty and zero-dimensional admission update — 2026-10-10

The historical empty-input restriction above is now removed through explicit
native recipe ownership. A chart-free dynamic generation result receives an
empty `NativeProgramDescriptor` only when it has no stochastic sectors, no
checking sources and exclusively literal zero exact coefficients. Compilation
then supplies an explicit empty saved-certificate set. There is no dummy radius,
fabricated causal factor or inferred scaleless contribution.

A genuinely zero-dimensional nonzero input still follows normal mapping and
retains its actual causal-factor metadata and exact value. The control with
prefactor 3 and constant F=2 at power -1 produces 3/2. Identically zero singular
denominators remain rejected by native input admission, including when their
prefactor is zero; an explicit zero prefactor with a valid denominator may
produce the ordinary empty exact result.

The focused empty filter passed **4/4 tests**, covering both dynamic recipes and
both generation modes through direct, cooperative and streamed generation,
native compilation, binary restoration and `always` binding. Root-free results
require no sampled pilot arguments and retain their selected mathematical recipe.
The existing fixed-empty regression also passes. Evidence is
`target/contour-dynamic-empty-tests.log`. These controls do not replace the
separate public endpoint, threshold and physical dynamic-contour gates.

## Public dynamic scientific gate — 2026-10-10

`crates/fastsecdec/tests/contour_dynamic.rs` now exercises the ordinary public
generation, compilation, binary restoration, runtime binding and evaluation
interfaces. All **3 tests passed, with none ignored, in 1.75 seconds** against
the current native owner dependencies. The log is
`target/contour-dynamic-public-analytic-root-fix.log`.

The endpoint controls evaluate the complete complex Laurent vector of
`(1+i) x^(-a-eps)/(1-2x-i0)` for a=1 and a=3. Polynomial/symbolic/Taylor,
polynomial/numerical-dual/IBP, sign-aware/symbolic/IBP and
sign-aware/numerical-dual/Taylor give the independent references obtained from
the native-verified recurrence `J_a=1/(1-a-eps)+2 J_(a-1)`:

- `J_1=-1/eps+i*pi+O(eps)`;
- `J_3=-4/eps-5/2+4*i*pi+O(eps)`.

The test multiplies the full reference vector by `1+i`, completes checked
preflight and then integrates restored native evaluators on 8192 identical
midpoint coordinates with production validation off. All eight combinations
passed their original declared tolerances; no tolerance was loosened. This
includes actual native deferred numerical-dual execution and higher endpoint
derivatives through the public kernel interface.

The threshold control uses `(1/eps) integral (1-5x(1-x)-i0)^(-eps) dx`. With
`beta=sqrt(1/5)`, its pole is real 1 and finite term is
`2-beta*log((1+beta)/(1-beta))+i*pi*beta`. Both constructions pass; native
`always`, `pilot` and `off` preserve identical sample values and mathematical
identity, while only `always` increments production validation counts. The
stationary point x=1/2 explicitly exercises the negative-real causal lower lip.

The first higher-pole attempt exposed a 3322-bit strict native root refinement
being limited to 256 iterations. The corrected runtime uses the exact native
reciprocal-square-root solution for a structurally quadratic envelope and a
precision-scaled finite allowance for genuinely higher-degree envelopes.
Independent owner/native tests retain tracked uncertainty, higher-degree
zero-centred coefficient slots and strict bracket termination. This correction
changes numerical execution, not the generated mathematical integrand.

These analytic controls establish the first public dynamic admission gate. They
do not establish full physical one-/two-/three-loop validation, portable/WASM
execution of this new gate, or a general variance/performance advantage.
