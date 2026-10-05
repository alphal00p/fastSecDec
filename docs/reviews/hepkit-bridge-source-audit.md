# Independent HEPKit bridge source audit

This independent review on 2026-10-05 covers the in-progress community bridge
in `DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/symbolica-community-notebook`:
`src/fastsecdec/{mod,input,generation,kernels,session,status,error}.rs`,
`python/symbolica/community/hepkit/fastsecdec/__init__.py`, the four builders in
`examples/hep/fastsecdec_inputs.py`, and their two Python test modules. It also
checks the corresponding native generation/status, QMC submit/retry, replay and
artifact ownership boundaries. The reviewer did not edit bridge implementation
or execute scientific code. The bridge owner retains fixes.

This is source review, not acceptance of the pending native/Python/portable
runtime gates or of browser responsiveness. The root coordinator will append
the executed results and independently review the requested fix.

## Finding requiring a fix

**P2 — record interruption consistently across package boundaries.** In the
reviewed `session.rs`, `step()` cleared its prior stopping reason, then used `?`
directly for the pre-package `py.check_signals()` and post-acceptance Python
observer call. A `KeyboardInterrupt` at either boundary therefore escaped with
`stop_reason=None`. An interrupt detected inside the weighted callback instead
set `StoppingReason::Cancelled`. The resulting snapshot/checkpoint depended on
the arrival boundary even though each interruption stopped the call.

Requested repair: preserve the original `PyErr`, record `Cancelled` for
`KeyboardInterrupt` at both outer boundaries, and preserve the accepted prefix.
Ordinary observer exceptions must remain their original types and must not
become numerical failures. Add a deterministic observer-`KeyboardInterrupt`
control checking the retained accepted package and checkpoint/resume. The
finding was delivered to the bridge owner and coordinator. The coordinator
reviewed the repair: both boundaries now record `Cancelled` for
`KeyboardInterrupt`, preserve the original exception and retain accepted work.
Other Python signal-handler errors preserve their provenance instead of becoming
numerical failures. The deterministic observer-interrupt test checks eight
accepted points, incomplete coverage, saved cancellation and successful resume.
Source review is closed; execution through the installed bridge remains pending.

## Native ownership and ecosystem reuse

`Integral` accepts existing `PyFeynmanDiagram`, `PyKinematics` and Symbolica
`PythonExpression` objects. It obtains the guarded complete native diagram,
clones it into the native `Arc` owner, and passes native atoms/kinematics into
`GraphIntegral`. Selected-subgraph rejection remains in HEPKit. Powers use
native stable `EdgeId`; scalar bindings, auxiliary momenta, measure and tensor
dimension remain in existing native APIs. No diagram/DOT reparser, physical
graph type, algebra engine, routing convention or numerical conversion layer
was introduced at this boundary.

The builders use HEPKit's ordinary DOT/model readers for the existing fixtures.
Their display-only numerator helper multiplies the native numerator, projector,
numerator prefactor and overall factor before native `simplify_algebra`, `to_dots`
and kinematic application. It is not fed back as a second weighted integral
input. Floating form controls become the exact binary rational represented by
the model card's f64 value; this is input conversion rather than a new algebra
implementation. The rank-two and sunset tests compare native tensor/dot
expressions and the coupled native denominator.

Generation calls `ParametricIntegrand::from_graph` and the public native
`generate`, retaining the native domain guard. Compilation uses the native
progress API and ordinary backend. The new kernel byte methods delegate to
`artifact_bytes`/`KernelSet::from_bytes`; they do not implement another program
format or execute serialized machine code. Generation, compilation and artifact
loading remain synchronous; signal responsiveness is limited by native callback
boundaries. This is explicitly documented, and is not browser responsiveness
evidence.

The saved `output/diagnostics/bridge-{root,community,portable}-identities.json`
records each contain a single owner for graph, model, kinematics, Symbolica,
Graphica, Linnet, Spenso and Numerica. Root/community use native FastSecDec;
the standalone portable consumer uses only its portable feature. Root/community
also retain the native one-loop master provider. These metadata observations
establish dependency identity, not successful execution of migrated scientific
controls. Existing one-loop master and numerator-reduction controls remain the
reference owners; their migration test handoff was requested from the bridge
owner, reusing built targets rather than creating a separate rebuild campaign.

The subsequent unified-owner native gate passes 42 tests, including five scalar
master and five numerator-reduction comparisons. The coordinator verified all
eight bound source hashes and six report hashes in
`output/diagnostics/algebraic-domain-1/result.json`. Explicit workspace formatting
and all-target Clippy also pass. This closes the native migration checks, while
installed Python and portable-wheel execution remain separate.

## Execution, statistics and error boundaries

The Python caller chooses when to call `step(max_packages)`. The bridge owns no
background loop or worker pool. It lazily retains one native worker/evaluation
context and releases the previous context before switching sectors. Native
`QmcSession`, published lattice catalogues, periodization and weighted replay
perform the numerical work. The bridge's rule getter reports the actual native
enum, and the constructor exposes four native published catalogues. It does not
reimplement lattice generation, sector aggregation, covariance or convergence.

Native `KernelResultManifest::integration_problem(FullIntegral, ...)` supplies
the full problem, including exact coefficients and repeated real/imaginary
signed-order entries. `VectorEstimate` transports native means, errors and the
complete row-major covariance without padding or component reduction. Its
`meets` method delegates to native `Tolerance` and `VectorEstimate::meets`.
`None`, waiting coverage and statistical failure remain distinct from a zero
estimate. Detached frozen snapshot wrappers expose native stage, coverage,
precision and failure details.

The package transaction is correctly ordered in the reviewed source: prepare
native contexts; evaluate a complete weighted return; validate a candidate
merged replay state; submit through native transactional validation; then
replace the accepted replay state. An evaluation/submit failure drops the
active context and retries the issued task through native `retry`; it cannot
commit the rejected prefix or its replay state. Diagnostic counters deliberately
include evaluated failed attempts, as their public documentation states.
Observer calls happen after acceptance, so an observer exception cannot undo
an already accepted package. Ordinary Python exceptions retain their types.

The checkpoint envelope delegates the numerical session bytes to native
checkpoint/restore and persists the bridge-owned replay policy/states,
diagnostics and stopping reason. Restore binds the expected full integral and
validates each replay state against the native kernel/policy. It is a narrow
caller-state envelope, not a second accumulator or estimator. Native errors are
exposed as `FastSecDecError` with machine-readable stage; cancellation has its
own subclass. Native error messages and statistical-failure explanations remain
visible rather than being converted into successful zero results.

## Required execution evidence

The inspected tests cover native input identity, selected-subgraph rejection,
separate tensor/regulator dimensions, generation/compile cancellation, observer
exception type, partial coverage, checkpoint/resume, full covariance, native
artifact roundtrip and wrong-integral rejection. The analytic massive triangle
check and complex multiplier test verify normalization and the full real/imag
covariance transform; the other builders check signed Laurent layouts. These
tests are present, but this review did not execute them or infer their outcomes.

Before accepting the bridge milestone, retain the actual focused results,
including the interruption repair, migrated one-loop/reduction controls,
dependency/build identity and the native/portable backend distinction. Native
gg→HH execution and any notebook/browser feasibility judgment remain separate
gates. No new on-shell triple-box work follows from this audit.
