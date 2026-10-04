# Ignored native process-timing harness

`output/probes/process_timer.rs` is a Linux-only Rust diagnostic helper for
bounded paired process measurements. It is not linked into FastSecDec and has
no integrator or symbolic functionality. It links against the already resolved
native libc, serde_json and BLAKE3 dependencies; no package manifest or dependency
source was changed.

```
output/probes/process_timer --output NEW_DIRECTORY --timeout-ms 180000 \
  --grace-ms 5000 --evidence BUILD_EVIDENCE.json --hash INPUT.toml \
  -- PROGRAM ARGUMENTS...
```

The output directory must not exist. `invocation.json` records the exact argv,
resolved outer executable, working directory, inherited CPU/memory affinity,
explicit performance-related environment whitelist, build-evidence identity and
all `--hash` input identities. File identities use BLAKE3 with their canonical
paths and byte lengths. It also records the timer executable's identity. When
PROGRAM is an execution wrapper such as `taskset`, the scientific executable
and Python/native dependencies must remain explicitly bound by build evidence
or additional `--hash` inputs. Environment values outside the whitelist are
inherited but neither enumerated nor certified; credentials and licensing values
are not captured.

`stdout` and `stderr` preserve the child streams. `report.json` adds outcome,
wall/user/system CPU time, native RSS/page-fault/context-switch counters, timeout
signals and output hashes. It verifies that executable, evidence and inputs did
not change during the run. Exit codes are zero for success, 124 for a timeout,
one for a failed child or changed identity, and two for a harness error. A child
failure retains its actual exit code/signal in the report.

A blocking Linux `waitid(WEXITED | WNOWAIT)` observes child completion; wall time
ends immediately after that wait returns, before watchdog joining, cleanup,
hashing or report serialization. There is no completion polling delay. Scheduler
wake-up delay and spawn cost remain part of observed wall time. Native `wait4`
then reaps the child and supplies rusage. `ru_maxrss` is retained in Linux KiB;
it is not a simultaneous aggregate process-tree peak and may report zero for a
very short child. CPU accounting includes only descendants reflected by native
wait semantics, not guaranteed orphan accounting.

The child receives its own process group atomically through the standard Rust
`CommandExt::process_group(0)`. A separate condition-variable watchdog sends
SIGINT at the deadline and SIGKILL after the requested grace. The leader remains
unreaped until group cleanup and monitor shutdown, preventing reuse of its PGID
while signals are sent. Any group members remaining after normal leader exit
are also killed. This bounds descendants which retain the inherited process
group; it is not a cgroup sandbox for a program deliberately escaping through
`setsid`/`setpgid`. A deadline race is conservatively classified by the completion
observation. The harness never signals unrelated host workloads.

Independent source review by the generation owner found no lifecycle blocker.
Three controlled tests passed: successful child/native accounting; nonzero exit
with retained stderr; and a SIGINT-ignoring child plus descendant terminated by
the owned group watchdog. The last check accepts an absent or killed-zombie
child, without claiming to reap a descendant owned by another parent. An actual
CLI invocation of the successful control also passed. Evidence is in
`output/probes/process-timer-tests.log` and
`output/probes/process-timer-cli-control/`. The initial test's assumption of a
strictly positive RSS counter was corrected after the native kernel returned
zero for a roughly one-millisecond child; that counter is preserved honestly.

The tested executable SHA-256 is
`5ff6d1ce5a8363c872a5a9f33139a1d7ece6f72770683aa9dc83300ca1af9ba9`;
its source SHA-256 is
`616a6482064e7a867cc293cdd0db2fe06a47cbd1a14f46e3e8d516e5a01693bb`.
The six large status-cadence runs predate this helper and retain their separately
disclosed 50 ms polling-wrapper precision; they have not been relabelled.
