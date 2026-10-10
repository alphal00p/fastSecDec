# Contour operational report transport

2026-10-10, after `a3d97cf`. This slice transports the already accepted native
`ContourRuntimeReport` through existing integration diagnostics and CLI status.
It adds no root algorithm, numerical observer, sampling loop or worker pool.

## Ownership and phases

`EvaluationDiagnostics.contour_runtime` is an optional boxed native
`ContourRuntimeDiagnostics`. Its adaptation and production reports each retain
the original preparation, exact, independent-pilot, evaluation and conditioning
categories. Recording and merging use the native checked, atomic report merge.
An absent report means observations were not available; it does not assert zero
work. The surrounding diagnostics now derive `PartialEq` rather than `Eq`, as
native range observations contain floating-point values.

The ordinary CLI drains each weighted numerical owner after a batch attempt,
before propagating success or failure. These deltas enter both existing
operational views and checkpoint diagnostics. Original binding and contour-pilot
owners are drained before sampling clones reset their private histories. This
keeps setup work visible without counting it as integration points.

Serial jobs carry the selected observation mode through the existing worker
request. Workers drain setup and batch owners and send their existing cumulative
task metrics. The coordinator replaces live progress; it admits a completed task
once or retains the last received report once when a reservation is abandoned.
Rejected work contributes no accepted statistical points. An abruptly terminated
worker can only report work already sent over IPC; unreported work is not
reconstructed or presented as zero.

All modes continue through existing QMC, adaptive QMC, Havana per-sector and
discrete-grid paths. Libraries remain caller-driven. Human status and JSON expose
native counters and ranges without evaluating a kernel.

## Configuration and identity

`integration.contour_diagnostics` and CLI `--contour-diagnostics` accept the native
`disabled` and `aggregate` choices, with Disabled as default. Workers check that
immutable mode once per batch before draining. There is no new per-point hook.
Mode changes are allowed between checkpoint invocations and excluded from the
mathematical, RNG, accepted replay and checkpoint compatibility identities.
Serial resident owners cannot change mode during one immutable invocation.
Callers that change a retained native owner's mode must explicitly drain old
history; the mode getter is not a claim that previous observations vanished.

Ordinary adaptive QMC recommends its allocation using measured worker costs.
Independent timed runs may therefore choose different allocations when
instrumentation changes those costs. The mode-equivalence regression restores
one native checkpoint and extends its frozen production allocation identically
for both modes. It compares the complete mean/error/covariance vector and point
count exactly for the same assigned work. No numerical tolerance replaces that
equality requirement. Fixed-allocation ordinary and serial routes also retain
bitwise full-vector equality across modes.

## Verification

The workspace all-target check passed. Two native status tests passed, covering
phase separation, old-record decoding, and atomic overflow without partial
diagnostic mutation. The CLI bin gate passed 107 tests with five explicit
subprocess fixtures ignored, including actual failed/discarded Eager and JIT
matrix work, unchanged outputs and a second drain returning no report.

The first actual-process fixture used a 128-point request with a Kuo rule whose
minimum is 1024; the fixture now explicitly selects the published HkknAlpha3
rule. Its initial cross-run adaptive equality assertion also exposed the intended
time-budget allocation distinction above. After correcting the test contract,
the two process controls passed in 6.42 seconds: nine supported ordinary/serial
sampler routes, adaptation/production attribution, mode changes on completed
checkpoints, frozen adaptive continuation and malformed-mode rejection.

The additional actual serial interruption control compares the last received
live cumulative report with the final operational report and persisted
diagnostics, while asserting zero accepted sample points. It passed together
with the two process controls in 6.94 seconds. The full CLI all-target gate passed
173 tests with eight explicit ignored fixtures across 27 executables. Combined
with the two native tests this is 175 distinct passing tests for this gate;
repeated focused runs are not added to that count. Six tests are newly introduced
by this slice (two native, one CLI unit, three process controls). Strict workspace
all-target Clippy passed in 8.90 seconds and formatting/diff checks passed. The
lint gate required one equivalent CLI-only struct-initializer change; it changed
no native or binding production source. The final focused rerun after that style
correction passed the same 107 CLI unit tests (five ignored) in 13.41 seconds and
three process tests in 7.22 seconds. These repeated tests are not additional
distinct acceptance counts.

The independent ecosystem/source review is recorded in
`contour-operational-transport-audit.md`. Root separately verified the immutable
candidate1 source archive
`9950910602b5c5fe040f30aa2a3d388ef8c28f63a4f6b7d11f02d0eec6d77c12`:
249 installed native Python tests passed in 78.45 seconds and 138 actual Pyodide
tests passed in 16.32 seconds. These include the thin phase/report getters,
Havana phase freeze and changed-mode checkpoint controls. This host evidence is
separate from the native/CLI gates; later test-only CLI corrections do not change
the captured production source.
