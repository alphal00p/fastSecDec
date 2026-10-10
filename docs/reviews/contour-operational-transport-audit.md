# Operational contour diagnostics transport audit

2026-10-10. Independent source review after `a3d97cf`; runtime, CLI and Python
source remain with their assigned owners. Executable gates are separate.

## Findings

No numerical or ownership blocker was found in the reviewed transport. The
native optional boxed `ContourRuntimeDiagnostics` records adaptation and
production separately, each retaining native preparation, exact evaluation,
independent pilot, evaluation and conditioning categories. Missing historical
or disabled observations remain `None`; they are not presented as zero work.
Checked merges are atomic on counter overflow and do not modify accepted sample
counts.

Ordinary CLI and Python evaluation adapters drain native reports after attempted
batch evaluation and before propagating failure. Thus actual work from failed
or discarded matrices survives without pretending those rows were statistically
accepted. Draining prevents a later batch from recounting that delta. The CLI
operational ledger and persisted diagnostics are separate views of the same
received data, not inputs to one another's totals.

Serial worker progress carries the current cumulative task report. The
coordinator replaces live progress, admits a completed report once, clears its
live copy, and preserves the last report once when an unaccepted task is
abandoned. The returned numerical result, replay state and diagnostic admission
are prepared together. Initialization drains preparation/exact/pilot work before
moving the native evaluator into the resident context. Reloads can legitimately
repeat this work; the report describes executions, not unique sectors.

Python getters clone native frozen DTOs. The sampling sessions reuse the native
evaluation adapter; there is no Python root observer or statistics algorithm.
Havana explicitly preserves only the phase-labelled operational history when
resetting ordinary phase statistics. Originating KernelSet reports remain
distinct from independent sampling-context work. Display and JSON access do not
run evaluators or sampling.

Diagnostics mode is explicitly excluded from ordinary and serial checkpoint
settings identity. No RNG, reservation, sample weight, covariance, physical
binding or accepted replay identity is changed by this transport. The disabled
adapter checks an enum before invoking native report draining; native disabled
callback behavior remains the previously audited owner implementation.

## Boundaries and verification

The serial resident settings fence does not include diagnostics mode. This is
sound for the current immutable invocation: every job in that worker's lifetime
has the same mode, and a resumed invocation creates fresh workers. If live mode
changes are added later, the resident owner must explicitly remap or reject them.

An abruptly killed worker can only contribute the last received IPC report;
unpublished work cannot be reconstructed. The current code does not invent that
missing history. This is an operational observation boundary, not loss or
acceptance of numerical samples.

Reviewed sources: native `status/contour_runtime.rs`, `status/diagnostics.rs`;
CLI `driver/execution/{mod,observations}.rs`, `driver/serial/{worker,coordinator,
checkpoint}.rs`, `driver/checkpoint.rs`, `isolated.rs`, and main preparation;
Python `execution.rs`, `mc/session.rs`, `status/diagnostics.rs` and
`contour/runtime.rs`. Leaf binding all-target/stubgen check and strict Clippy
passed. Installed candidate-1 source content SHA256
`9950910602b5c5fe040f30aa2a3d388ef8c28f63a4f6b7d11f02d0eec6d77c12`
passed native Python 249/249 in 78.45 s and actual Pyodide 138/138 in 16.32 s,
including the three new status cases on both hosts. The native status owner
reported 2/2 focused tests and the full CLI 173 passed/8 ignored, followed by
strict workspace all-target Clippy, formatting and diff checks. The final
107-unit/3-process recheck also passed. See the owner's
[status gate record](contour-runtime-status.md) for commands and source scope.

The final adaptive-resume test correctly uses native `QmcSession::restore` and
`extend_production_shifts` to compare modes on the same frozen allocation;
independently timed adaptation would not promise identical sampling work.
The actual SIGINT test also passes source review: the last received cumulative
worker report is preserved in the final report and checkpoint, while the
interrupted reservation still has zero accepted points. The reader retains the
last report after the interrupt trigger, avoiding a false equality against an
earlier progress snapshot. A hard kill can still lose untransmitted work, as
documented above.
