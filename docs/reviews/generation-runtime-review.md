# Runtime kinematics and threshold policy integration review — 2026-10-06

Independent review by the generation-dispatch implementer; runtime parameter
binding and threshold policy were implemented by other agents. This review
covers the current request, not the deferred full scientific test migration.

## Native ownership and API reuse

Kinematic scalar products continue to enter native HEPKit `Kinematics` and the
existing integral-family preparation. A declared runtime symbol is a Symbolica
`Symbol`; the native evaluator builder receives it after the integration
coordinates. There is no alternate scalar algebra, physical graph, momentum
parser, numerical evaluator or special-function implementation. Point-file
expressions and dependencies reuse `resolve_scalar_bindings` and Symbolica's
existing evaluation APIs.

The same immutable exact evaluator is mapped through the existing native O2 or
portable interpreter backend. Integration still calls caller-owned workers;
binding a physical point does not generate, expand, compile or sample sectors.
The public parameter/bind APIs are backend-independent and available to later
Pyodide bindings. An actual Pyodide wheel execution is separate from the portable
host check and is not established by this review.

Existing native one-loop master and numerator-reduction reference providers
are unchanged. This change does not reproduce those numerical algorithms or
add new reference implementations; the current algebra/probe checks address
parameter transport and evaluator semantics rather than a new one-loop science
gate.

## Bound-state and numerical boundaries

Runtime symbols have a complete ordered schema; duplicate coordinate/runtime
symbols are rejected by evaluator construction. Unknown, missing and nonfinite
bound values are errors. Rebinding derives a numerical identity from the
immutable template identity and the ordered binary64 values, so point changes
cannot silently reuse a previous numerical checkpoint.

Real and complex evaluators append scalar values after integration coordinates.
Only integration coordinates have unit-cube restrictions and contribute to
endpoint cancellation estimates. Native precision rescue receives all evaluator
inputs, including runtime scalars. Worker clones retain the bound values and
precision state. Analytic exact offsets use a separate zero-dimensional native
evaluator and are evaluated at the same bound point before sector workers are
made available. An unbound exact vector is nonfinite, so manifest validation
rejects an integration session even for a zero-sector result; direct evaluator
and weighted-replay entry points report unbound state explicitly.

A failed parameter rebind does not replace the accepted sector values, exact
vector or bound numerical identity. The exact-offset worker is reset on the
next binding attempt. Portable kernel bytes remain the reusable unbound
template; saving a numerical result/checkpoint is a separate operation. The
review requested explicit template-identity documentation to make this
boundary clear.

## Threshold policy

Production generation no longer performs coefficient-sign, face or interior
threshold certification. Metadata records the caller's responsibility and the
chosen domain. The retained residual check only verifies the algebraic
monomial-extraction invariant: its symbolic constant coefficient is nonzero.
It does not evaluate a physical point or certify threshold freedom.

The real-domain numerical evaluator still reports undefined/nonfinite values
and precision failures; the policy change does not substitute zero for failed
numerics. Historical sign-test helpers remain test-only while the user-requested
migration of old tests is deferred.

## Validation boundary

Source review found no new mathematical implementation or ownership duplication.
The owner probes report eager parameterized generation/compilation and runtime
evaluation under native O2 and the portable-feature host build. The reviewer
also ran a six-kernel, three-coordinate direct CLI input through physical and
native-named coefficient expansion at one and four workers. Within each route,
the two worker counts produced identical binary artifacts and content identities.
The status stream reported the actual worker allocation for every dispatched
stage. A separate active-worker cancellation of gg→HH at the mapping stage
exited nonzero and published neither sibling artifact. Scoped strict core/CLI
Clippy passed after the final dispatch changes. A controlling-terminal PTY run
also rendered the aggregate bar, elapsed/ETA, actual worker rows and RGB palette;
pressing `q` while workers were active exited nonzero without a signal fallback.
`NO_COLOR` remains respected. Raw probe cards/output live under ignored
`output/parallel-smoke/`, not tracked examples or tests. Dispatched stage timings
now measure coordinator wall intervals; local job durations are not summed into
wall-time fields. This document does not claim a new actual Pyodide run, full
gg→HH integration convergence, or the postponed all-example test gates.
