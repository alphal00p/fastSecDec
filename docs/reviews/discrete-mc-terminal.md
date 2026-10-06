# Discrete MC terminal and interruption lifecycle

The CLI dashboard and its terminal ownership were reviewed and probed on
2026-10-07. Numerical evaluation, integration statistics, RNG, and checkpoint
ownership are unchanged by this slice. The independent dispatcher review is in
[discrete-mc-dispatch.md](discrete-mc-dispatch.md).

## Failure mechanism

Raw terminal mode turns Ctrl-C into an input event rather than an OS SIGINT.
The former discrete MC coordinator waited for an entire worker batch before
reading another event; expensive precision rescues therefore made both the
screen and keyboard cancellation appear stuck. Separately, the former
conditional-default signal handler terminated immediately on a second SIGINT,
bypassing the dashboard destructor and leaving raw mode and the alternate
screen active. The former key matcher also treated an unmodified `c` as
cancellation and stopped reading keys once cancellation was set.

The worker coordinator now polls during a batch. The dashboard keeps polling
keys after the first cancellation, requires the Control modifier for `c`, and
handles both keyboard and OS interruption through the terminal owner. A second
Ctrl-C requests immediate exit after terminal restoration; it does not promise
a new checkpoint. The first interrupt remains cooperative and preserves the
existing pilot/production resume semantics.

## Reused native owners

The new CLI-only `display/terminal.rs` uses the existing Crossterm 0.29 and
signal-hook 0.3.18 dependencies. No library thread, numerical callback, or new
signal/terminal implementation was introduced.

* Crossterm owns saved termios and native enable/disable operations. The guard
  disables raw mode only if it enabled it. It records ownership before the
  compound alternate-screen/cursor write so a partial entry error is covered.
* Registered signal callbacks only update preallocated atomics. A blocking
  signal-hook `Signals` iterator wakes ordinary Rust cleanup code on Unix;
  neither terminal I/O nor a mutex is used inside a signal callback. A native
  API probe confirmed registration order before the pipe wakeup.
* The listener restores terminal state before forced exit, including while the
  coordinator is blocked in a native evaluation. Normal destruction closes the
  iterator, joins the listener, and unregisters the atomic callbacks. There is
  no polling thread or per-frame shell command.
* An installed panic hook restores the active terminal before invoking the
  previous reporting hook. The active owner is weakly held. Subsequent drawing
  checks the guard state, so a successfully caught panic cannot re-enter the
  restored terminal implicitly.
* Each complete frame holds stderr's native reentrant lock and rechecks the
  active state under that lock. Entry and restoration acquire stderr before
  the mode mutex. Cleanup therefore runs after an already-started frame, without
  permitting trailing cursor controls to undo restoration. The same-thread
  panic hook can reacquire stderr; another thread cannot invert the lock order.

The Windows branch retains restoration at coordinator polling boundaries;
there is no Unix signal iterator there. Only the macOS path was compiled and
run in this environment. SIGKILL and other exits which cannot execute cleanup
are outside this guarantee.

## Live work is observational

`IntegrationWorkerActivity` belongs to the CLI display. It records each worker's
batch, completed/planned in-flight points, current sector, and whether it is
preparing an evaluator. The full dashboard distinguishes preparation,
evaluation, and a finished batch awaiting admission. Compact and plain views
show the aggregate unfinished workload. JSON adds `in_flight_unaccepted`.

Accepted points, estimates, errors, covariance, and coverage continue to come
from the native integration snapshot. The overlay never adds unfinished points
to those fields. Recorded diagnostics are labeled separately because their
in-flight prefix is available only after a worker returns. The footer identifies
that pending interval rather than presenting those counters as current-worker
measurements. The original sector-coverage panel returns when no wave is active.

## Focused PTY evidence

The ignored probe sources and raw captures are under
`output/interrupt-review/`. The final source was compiled against the production
terminal owner and a copy of the production dashboard. A Python standard-library
PTY harness injected input or signals only into its own child processes.

All of these checks passed:

* Normal return, ordinary error, and panic-hook restoration each issued a
  matching alternate-screen leave, showed the cursor, and restored the saved
  terminal configuration. On macOS the kernel-maintained `PENDIN` transient bit
  was excluded from configuration comparison; echo, canonical input, signal
  handling, and all other saved settings matched.
* An outer owner enabled raw mode before constructing the dashboard. Dropping
  the dashboard preserved that raw mode, and the outer owner then restored its
  original state.
* Unmodified `c` left the process running; Control-C cancelled. A second
  Control-C exited with status 130. A second SIGINT or SIGTERM restored the
  terminal and exited with status 130 or 143 while the main thread was sleeping
  for 20 seconds.
* Full 120-column and compact 64-column modes restored the terminal. Plain and
  JSON modes emitted no alternate-screen controls and did not change terminal
  configuration. JSON preserved accepted points at zero while reporting eight
  in-flight, unaccepted points in a separate field.
* One hundred sequential interactive dashboard lifetimes produced 100 matched
  enter/leave pairs. Native process thread counts were one before and one
  afterward; cancellation handles each had strong reference count one after
  destruction, confirming callbacks did not retain them.
* A separate ordered-frame probe held the native stderr lock while a second
  thread attempted restoration. Restoration waited until the frame finished;
  the last cursor-hide preceded the alternate-screen leave. Same-thread
  restoration under an already-held stderr lock completed. A panic under that
  lock also restored the terminal before the host's unwind failure. All three
  screen entries had matching leaves, and the saved termios was restored.

In the final measured isolated run, first keyboard cancellation took 15–21 ms,
second keyboard cancellation 20 ms, and forced second SIGINT/SIGTERM 11–13 ms.
Plain/JSON cooperative signal completion took 13–22 ms. These are dashboard
probe timings, not bounds on native evaluation or checkpoint time. Actual ggHH
one/eight-worker acceptance is recorded separately by the root review.

The linked native stack on this host reports `failed to initiate panic, error
5` and aborts when trying to unwind even the caught-panic probe. The hook had
already restored the terminal before that abort. Consequently the test proves
pre-abort panic restoration; it does not prove caught-panic continuation on this
host. The independent source audit accepted the restored-state drawing gate.

Independent source review by the dispatcher owner found no actionable issue in
signal ordering, cleanup ownership, close/join/unregister, or repeated-key
handling. No permanent tests, existing gates, or other examples were changed.
