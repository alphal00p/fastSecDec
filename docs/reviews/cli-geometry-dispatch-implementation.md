# CLI-owned native geometry dispatch

`generate` and `run` now accept positive `--geometry-workers N`, defaulting to
one. The serial path is unchanged. Above one, the CLI owns a Rayon pool and
dispatches only native chart/cone jobs through `GenerationContext::new(0)`.
The library retains domain admission, plan ownership, opaque results, complete
coverage checks and canonical merging. No geometry algorithm, CAS operation,
integration setting, artifact schema or persistent cache is added.

The coordinator stays on the calling thread via native `in_place_scope` and
keeps at most N jobs active. Its bounded channel can drop progress observations
but retains each completion. Worker errors stay inside native completions so
canonical scientific error precedence is preserved. Worker panics become a
failed scheduling result; cancellation stops launching work and joins all
launched workers. Coordinator unwinding latches cancellation and drops the
receiver inside the scope body before Rayon joins, preventing a blocked sender
from stranding scope exit.

Only the coordinator polls or renders the terminal. Keyboard cancellation now
latches the existing shared flag, also read by native job callbacks. Status
distinguishes returned chart/cone jobs awaiting admission from native accepted
geometry counts. Existing display errors retain their original message. Scope
exit and the existing dashboard Drop retain thread and terminal cleanup.

The worker option is observational execution steering, excluded from numerical
and checkpoint settings. An already completed resumed run bypasses geometry
entirely. The immediate generate report records the requested geometry workers;
retained mathematical artifacts are identical to serial output apart from
generation timing observations.

## Executed checks

`output/cli-geometry-dispatch-cli-tests.log` records **49 passed, zero failed,
three ignored**, across 12 result summaries. This includes two new scheduler
lifecycle tests and two new process tests:

- Active native geometry cancellation and coordinator panic unwind join workers
  and leave the caller cache empty; worker panic transport and an empty stage
  also complete without stranded work.
- Serial and two-worker artifacts have identical ordered native kernels,
  metadata, exact contributions and content identities after excluding timings.
  The independent Mellin integral of `x^(-1+eps)/(x+y)` gives the complete
  `[-2,-1,0]` vector `[1,0,pi²/12]`, verified numerically after parallel `run`.
- Domain and sector-limit errors agree between serial and parallel execution,
  with no artifact written. Zero workers are rejected during argument parsing.
  Resume with a different geometry count preserves the complete estimate and
  checkpoint conventions.

Forced reverse-completion admission remains covered by the unchanged native
`parallel_geometry` and `generation_context` tests from the accepted library
gate. The CLI process comparison exercises its own bounded scheduler and
canonical final output; it makes no assertion about which OS worker finishes
first. No artificial timing hook was added to the production scheduler.

Two initial process-test assertion failures are preserved as
`cli-geometry-dispatch-process-tests-attempt-{1,2}.log`. They referenced existing
settings at the wrong JSON locations: numerical workers are intentionally absent
from checkpoint identity, and artifact integration settings live under
provenance. Only those test paths were corrected; scientific checks already
passed and no production change resulted.

Formatting and `cargo clippy -p fastsecdec-cli --all-targets --locked -- -D warnings`
pass; the only emitted dependency warning is the previously recorded upstream
unused Result. Frozen debug executable and reviewed source hashes are retained
in `output/diagnostics/cli-geometry-dispatch-build-1`. Num independently reviewed
the applied implementation without a finding.

The coordinator's real PTY gate also passed and was independently inspected.
`cli-geometry-dispatch-pty-2` used the hard four-loop input, observed active chart
and cone geometry, resized from 110x32 to 60x18 and back, then cancelled with
`q`. It exited unsuccessfully, restored identical termios, left the alternate
screen, showed the cursor and wrote no artifact. NO_COLOR produced no explicit
color sequences. The `pty-normal-1` and `pty-color-1` controls exited zero,
restored the same terminal state and wrote identical artifacts after excluding
timings; explicitly unsetting NO_COLOR restored colored output. Frozen source
and binary hashes pass after these checks.

PTY attempt 1 is retained: its watchdog used a background process group and
never reached UI execution before the 45+5-second bound. Adding the documented
GNU timeout `--foreground` option fixed the harness without changing the CLI.
No performance measurement or parallel symbolic-generation claim follows from
these tests.
