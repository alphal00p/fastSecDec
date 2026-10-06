# Discrete MC dispatch and terminal review

Date: 2026-10-07. Scope: CLI responsiveness for native discrete-sector Havana
sampling. No integration rule, precision policy, estimator, covariance reduction,
artifact mathematics, or permanent tests were changed in this slice.

## Reproduction and diagnosis

The previous release was preserved under ignored
`output/integration-hang/fastsecdec-before`. Separate runs used the existing
`output/gghh_double_box.fsd`, `examples/gghh_double_box/point.toml`,
`--method discrete_mc`, and worker counts 1 and 8. Every checkpoint and result
path was under `output/integration-hang`; the user's outputs were untouched.
After 12 seconds, a three-second native `sample` profile was collected, then
SIGINT was sent. Neither process returned within three seconds. Only the two
processes created by this probe were terminated and reaped, after 18.3 and
18.5 seconds respectively.

Both runs emitted only their initial pilot snapshot with zero accepted points.
Their worker stacks were actively evaluating
`WeightedEvaluationContext::evaluate_weighted` →
`ComplexKernel::evaluate_scaled` → MPFR arithmetic. The computation was not
deadlocked. The coordinator was blocked in
`pool.install(...par_iter_mut()...collect())` until a complete wave of native
4096-point batches returned. Worker callbacks did not check cancellation.
Raw terminal mode additionally turns Ctrl-C into an input event, which that
blocked coordinator could not read.

Ignored evidence: `baseline.py`, `baseline-w1.stderr`, `baseline-w8.stderr`,
and `baseline-w{1,8}.sample.txt` in `output/integration-hang`.

## Execution change and native reuse

`driver/execution/discrete_mc/wave.rs` uses the caller's existing Rayon pool and
`in_place_scope`, following the existing QMC coordinator pattern. There is no
library-owned pool. Native `HavanaDiscreteTask`, `HavanaDiscreteWorker`,
`HavanaDiscreteReturn`, and `HavanaDiscreteSession` still own sampling, RNG
identity, whole-batch statistics, adaptation and full-vector covariance.

The coordinator polls a completion channel with a 50 ms timeout. It continues
polling dashboard input while workers execute or drain after cancellation.
Callbacks check the local stop flag before each point and after lazy evaluator
preparation. A stop-on-drop guard also stops workers if coordinator I/O fails;
a caught worker panic sends a completion notification instead of stranding the
coordinator.

Original wave/slot assignment and submission order are preserved. Completed
batches are admitted in original task order. Incomplete batches do not submit
numerical results or update accepted replay state. Successful sibling batches
remain eligible for admission. Genuine evaluation failures remain failures;
only the explicit caller-stop path marks an aborted prefix. Diagnostic counts
still describe actual work, including discarded prefixes, as in the existing
QMC implementation.

CLI-only atomic observations expose each worker's batch, selected sector,
evaluator-preparation state and completed point count. The dashboard labels
these as **in flight, not yet accepted**. They are separate from the native
accepted-point count, estimate, covariance and checkpoint. No partial native
batch submission or invented live estimate is used.

A single native point evaluation or evaluator construction is not forcibly
preempted. The first interrupt requests cooperative cancellation; repeated
interrupt handling belongs to the terminal lifecycle owner. The poll interval
is not a hard upper bound on native-call completion or filesystem latency.

## Scientific controls

The ignored `scientific.toml` fixture uses six numerical sectors and the full
four-component real/imaginary Laurent vector for orders 0 and 1. It runs two
pilot iterations of eight 128-point batches, followed by eight 256-point
production batches. The same saved artifact and seeds were evaluated with the
preserved old executable and the new debug executable, separately at one and
eight workers.

For each worker count, the following match exactly:

- Mean vector and complete covariance matrix.
- Evaluation diagnostics and accepted replay state.
- The entire checkpoint after excluding observational worker/sector/elapsed
  timing fields, including native proposal, seeds, records and RNG state.

This comparison is old versus new at the same worker count; it does not assert
that arbitrary replay decisions are bitwise invariant across different worker
schedules. Evidence is `old-w{1,8}` and `new-w{1,8}` report/checkpoint/result JSON
under `output/integration-hang`.

The final release executable repeated both controls after the terminal source
freeze. It again matched the preserved old executable's mean, complete
covariance, evaluation diagnostics and checkpoint excluding only the three
timing fields above. Evidence is `final-release-w{1,8}.*` in that directory.

A separate production probe requested 100,000,000 points per batch and sent
SIGINT after observing positive in-flight progress. It completed cancellation
in 0.078 seconds with one worker and 0.075 seconds with eight workers. The last
observed unaccepted counts were 33,984 and 333,650. Both reports had zero
accepted production points, zero numerical failures, stopping reason
`cancelled`, and `checkpoint_saved`. Both native checkpoint `records` arrays
were empty: no partial batch entered production statistics. Evidence is
`cancel.py` and `cancel-new-w{1,8}.*` in the same ignored directory.

Native checkpoint hole/reissue and successful-sibling covariance checks were
independently reviewed by the parameters agent. Native restore retains the
proposal and seeds, omits pending reservations, and reissues missing whole
batches with their original identity.

## Independent terminal source audit

The dispatch author independently reviewed the terminal owner's
`display/terminal.rs`, `display/integration_activity.rs`, and dashboard wiring.
The source checks establish:

- Signal-context callbacks use atomics only. An ordinary blocking
  signal-hook iterator performs terminal cleanup for a repeated OS signal.
  The owning registry's ordered action slots support the registration order
  used for atomic state changes before listener wakeup.
- Raw-mode ownership and alternate-screen entry are recorded before fallible
  subsequent work. Normal return and setup errors unwind through restoration.
- The panic hook restores terminal state. Later generation/integration renders
  detect that the terminal is inactive and use plain output rather than drawing
  into the restored screen.
- A complete TUI frame and restoration hold the same native reentrant stderr
  lock. Both generation and integration recheck terminal activity after taking
  that lock; restoration clears activity before waiting for an active frame.
  Thus a racing frame either completes before cleanup or skips drawing. Entry
  and restoration both acquire stderr before the mode mutex, with no reversed
  ordering in those paths. Rust's `io/stdio.rs` confirms the shared
  `ReentrantLock`, allowing same-thread panic-hook cleanup during a frame.
- Keyboard polling remains active after the first cancellation request, drains
  a bounded number of events, and recognizes Ctrl-C using its modifier and
  press event. Repeated interruption restores the terminal before exiting.
- The blocking signal listener is closed and joined on ordinary drop; no busy
  polling thread or numerical worker pool is introduced.
- Existing checkpoint writes use the atomic writer, so forced termination does
  not deliberately truncate the previously published checkpoint.

The final frozen source was reread, including the distinction between batches
still evaluating and those awaiting admission, and the explicit pending label
on diagnostics from in-flight work. No actionable source finding remained.
The terminal owner's final PTY suite passed normal/error cleanup, pre-abort
panic-hook restoration, outer raw-mode ownership, repeated keyboard and OS
interrupts, full/compact/plain/JSON modes, and 100 sequential dashboard
lifetimes without retained listener threads or callback references. Isolated
first-key cancellation took 12–16 ms; repeated SIGINT/SIGTERM cleanup took
11–13 ms even while the main thread was sleeping. These timings measure the
terminal harness, not native evaluation or checkpoint latency. See
[discrete-mc-terminal.md](discrete-mc-terminal.md) for evidence and the host's
native panic-unwind limitation. Windows-specific signal behavior was not
exercised on this macOS host.

After the draw/restore synchronization correction, the independent source
review and owner's full lifecycle suite were repeated. A focused race probe
confirmed that cross-thread restoration waits for an active frame, the last
cursor-hide precedes the final alternate-screen leave, and all three terminal
lifetimes restore their saved settings. Same-thread restoration under the
stderr lock completed. A panic under that lock restored the terminal before
the host's native panic-runtime abort, so lock reentrancy was exercised without
claiming successful caught-panic continuation. This correction changes terminal
locking only; the numerical release controls above remain applicable.

## Checks and limits

Native CLI `cargo check` and strict scoped Clippy passed after the dispatcher
and display API were connected; `git diff --check` passed. Existing test files
and other examples remain unchanged as requested.

The costly native precision-rescue path was diagnosed separately. This patch
does not weaken its conservative underflow checks or claim a numerical speedup.
Large user-requested jobs can still take substantial compute time; they now
have live observational progress and cooperative cancellation during a batch.
One-loop masters, reduction and reference providers remain unchanged; this is
an execution/presentation change, not another numerical implementation.
