# Generation memory monitor (2026-10-07)

The generation dashboard now shows the current FastSecDec process RSS and its
highest observed RSS, alongside machine-wide RAM used, total and available.
The same measurements appear in compact terminal mode, plain status and JSON
status. The RAM panel has its own colors and obeys `NO_COLOR`; its height leaves
the existing worker activity table scrollable.

## Owner and reuse evidence

Memory sampling lives entirely in `fastsecdec-cli::display::memory`. It reuses
[`sysinfo` 0.39.6](https://docs.rs/sysinfo/0.39.6/sysinfo/index.html), with default
features disabled and only `system` enabled. Its Rust 1.95 minimum is below the
repository's Rust 1.96 requirement. No core, Python binding or portable evaluator
dependency was added. The dependency's optional multithread feature is disabled;
the monitor starts no background pool or thread.

The public APIs were checked in documentation, implementation and owner tests
before adding the adapter:

- `System::new`, `refresh_memory_specifics` with RAM-only `MemoryRefreshKind`,
  and `refresh_processes_specifics` with `ProcessesToUpdate::Some` for the current
  PID, memory-only `ProcessRefreshKind`, and `without_tasks`.
- `Process::memory` is explicitly resident-set bytes. On macOS the owner reads
  native `proc_taskinfo.pti_resident_size`; no shell command or process-tree
  enumeration is required.
- Owner tests in `tests/process.rs` exercise selective memory refresh and
  disabling Linux task collection. The adapter does not inspect or add thread
  RSS values: all workers already share the monitored process address space.

A focused Rust probe touched 32 MiB from each of two live worker threads, then
queried exactly one PID. RSS increased by approximately 67.3 million bytes.
Combined machine-RAM/current-PID refreshes took 76–186 microseconds in these
macOS observations. This is local evidence, not a cross-platform performance
guarantee. No missing native OS API was replaced with a custom parser.

## Measurement semantics and scheduling

The existing dashboard polling loop calls the monitor. OS queries occur at most
once per 500 ms; intermediate frames and JSON messages reuse the cached sample.
There are no shell subprocesses, new timers, callbacks into the numerical core
or measurements on the evaluator's hot path. The monitoring object remains
uninitialized with respect to OS data until generation requests its first
sample.

RSS is physical resident memory for the complete FastSecDec process, including
workers and shared native libraries. It is neither a sum of thread allocations
nor total allocated/virtual memory, and it excludes child processes. The
**observed peak** is the maximum of sampled process readings since this dashboard
began monitoring. It is not an operating-system high-water mark: short-lived
peaks between samples can be missed, including in runs shorter than 500 ms.

System RAM values use native OS accounting from the library. Available memory
can include reclaimable pages and is explicitly labeled `(OS)`; it need not
equal total minus used. In particular, current macOS accounting can overlap the
used and available categories. These are machine-wide readings, not a container
or scheduler memory limit. They are presented separately rather than added to
the process reading.

Unsupported systems or an unavailable process reading produce `unavailable`
text and JSON `null` values, never a fabricated zero. The previous observed peak
is retained if a subsequent process query is unavailable. JSON's `memory`
object exposes optional byte-valued measurements plus `sample_interval_ms` and
`sample_age_ms`, so consumers can distinguish a cached reading. Display units
are binary MiB/GiB/TiB.

## Focused validation

The temporary probe used the implemented monitor to confirm the 500 ms cache,
refresh after a 510 ms interval, RSS growth from both worker threads, monotonic
observed peak, and unavailable-value rendering. It also checked that the native
query enumerated only one PID.

`cargo check -p fastsecdec-cli --locked` and strict Clippy for the CLI binary
passed. A native runtime-parameter bubble generated successfully with 31 JSON
status messages, each containing valid RAM fields, sample interval and age.
Its plain report and status JSON contained no ANSI escapes.

Actual pseudo-terminal generation runs passed with a 120×34 full dashboard,
64×18 compact dashboard and `NO_COLOR` full dashboard. Both memory scopes and
the observed peak were present in all modes; color was present only when
enabled. The complete generation path remained successful. Coordinator ETA now
remains unavailable without a matching live workload, so earlier parallel
preparation time cannot be charged again to subsequent serial comparisons.

Probe sources and terminal/status captures are retained only under ignored
`output/generation-memory/`. Existing tests and other examples were not edited.
Linux/Windows runtime behavior and container-specific limits were not tested on
this macOS host; those OS adapters remain owned by the reused library.
