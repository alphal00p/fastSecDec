# Minimal CLI adoption of caller-owned geometry dispatch

This accepted proposal connects the native dispatch entry to the existing CLI
executor, status and cancellation owners. The implementation and focused gate
are recorded in `cli-geometry-dispatch-implementation.md`; no speedup is claimed.
Free library generation and both existing context methods retain their current
serial defaults.

## Entry and scope

Add a positive `--geometry-workers` option to `generate` and `run`, defaulting
to one. Keep `--workers` as numerical integration steering. This avoids silently
making an existing integration override change symbolic preparation. There is
no new card field, numerical setting, checkpoint field or artifact-identity
input. A resumed run loads its existing artifact and starts no geometry pool.
The requested geometry count should be visible in parallel-generation status
and the immediate generate report as execution information, not as mathematical
provenance or an integration-worker count.

At one worker, retain the current cache-free free `generation::generate` route
and progress stream. Above one, construct a CLI-owned Rayon pool using the
already installed dependency and `ThreadPoolBuilder`, then call
`GenerationContext::new(0).generate_with_dispatch`. Capacity zero deliberately
retains no fan between unrelated CLI commands; this slice does not invent a
persistent cache. Native admission, support extraction, both work stages,
canonical merge, domain reassessment and later symbolic stages remain the
existing owners. Only native chart/cone jobs enter worker threads; input
parameterization, Atom operations, mapping, symmetry, subtraction, Laurent
generation and compilation remain on the existing caller path.

## Bounded dispatcher and coordinator

Use a small private `generate/geometry_dispatch.rs` adapter, not a reusable
task framework. The callback consumes native lazy `GeometryJob` iterators,
keeps at most the selected number of jobs in flight and returns their opaque
`GeometryCompletion` values. It must never manufacture a fan or inspect private
maps. Keep scientific failures in native completions and finish the prescribed
stage so native canonical error precedence remains authoritative. Empty stages
return an empty completion vector; zero-input integrals dispatch nothing.

The installed Rayon 1.13 `ThreadPool::in_place_scope` executes its coordinator
closure on the calling thread and joins all spawned tasks before returning.
It has no `Send` requirement on that coordinator closure. This existing API
lets the terminal owner continue polling while workers run, without moving a
dashboard or borrowed lazy iterator onto a worker and without a library pool.
Native jobs themselves are already `Send + Sync` and own shared input handles.

A bounded standard channel carries two private message kinds: tagged native
job progress and mandatory job completion. Worker progress uses nonblocking
`try_send`; full progress queues may coalesce/drop observations. Completion
messages must never be dropped. The caller drains messages with a short timeout,
launches the next lazy job only when a slot is free, and joins the scope before
returning on every path. Do not use a blocking `pool.install` around the entire
stage, which would prevent ordinary terminal polling. Completed results still
need storage until native admission; an in-flight limit is not a byte-memory
bound for the full prepared fan.

Worker completion must be accounted for even if a worker panics. A narrow
caller-side panic transport or completion guard prevents the coordinator from
waiting forever; after cancelling and joining outstanding work, preserve the
failure as a non-success CLI outcome. Do not convert it into a successful
completion or discard an earlier presentation/scheduling error. This is executor
lifecycle handling, not recovery from a failed scientific operation.

## Status, cancellation and presentation ownership

Reuse `GenerationSnapshot`, the existing dashboard renderer and the same
signal-handler flag. Add only a private cloneable cancellation handle for the
CLI worker adapter. When coordinator polling sees `q`/Escape/current cancel
keys, latch that existing flag so worker callbacks observe the same request.
Workers only read the flag and send small native progress records; they never
read terminal events, print, or mutate the dashboard. Cancellation remains
cooperative at existing native geometry callbacks, not preemption inside an
exact arithmetic call.

The coordinator pumps status/cancellation at approximately the existing 40-ms
terminal cadence, including timeout wakeups with no worker messages. Keep
existing plain/JSON policies and force stage-start/stage-end/failure updates;
do not emit unbounded JSON for every native local callback. Local job progress
is explicitly tagged by chart/candidate and must not masquerade as global
accepted rays, constraints or sectors. A stage display may report returned jobs
out of scheduled jobs with detail stating that native admission is pending.
Only native final completion supplies the accepted geometry map count, and the
existing later phases retain their usual semantics.

Both the ordinary generation observer and dispatcher need exclusive access to
the same caller-side presentation state. Keep this sharing local to
`generate.rs` (for example one `RefCell` around a small borrowed UI state, used
only on the coordinator). No dashboard lock or `RefCell` crosses into workers,
and no callback reenters while a presentation borrow is held. Existing display
errors retain their original message and trigger the shared stop flag.

On cancellation or presentation/scheduling failure, stop launching jobs, signal
active workers, drain/join every launched task, and return failure. No partial
artifact or geometry checkpoint is written. Native complete-cache admission
stays unchanged; this zero-capacity CLI caller retains nothing. Drop restoration
of the existing dashboard still owns raw mode, alternate screen and cursor.

## Files and focused acceptance

Expected changes are limited to command arguments/wiring in `main.rs`, the
generation adapter and small private scheduler module, a private dashboard
cancellation handle, relevant CLI process tests and concise usage documentation.
No kernel, estimator, geometry algorithm, artifact codec, integration driver or
new dependency is needed. Pool creation is included in total command elapsed;
native geometry timing continues measuring its existing phase. Do not claim a
speedup or comparable phase boundary without a later isolated measurement.

Acceptance should use a small native input that exercises both chart and cone
jobs. Compare serial and two-worker generation through native artifact load:
same complete coefficients/layout, exact offset, retained chart/map metadata,
kernel content identity and analytic integral. Exercise `run` as well as
`generate`, while keeping numerical worker settings unchanged. Reuse the
existing domain/resource-limit process cases with the parallel option, and
check no artifact is created on failure.

One focused scheduler test should force reversed native completion arrival and
cooperative cancellation while jobs are active, checking all tasks joined and
that only native opaque completions reach admission. A synthetic/pure-status
PTY control can verify `q`, resize and cleanup during the receive loop without
depending on a slow scientific case. Cover empty stages and a scheduling failure
that would otherwise strand the coordinator. Retain existing serial CLI gates;
avoid a broad repeated scientific campaign for this caller-only change.
