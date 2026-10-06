# Discrete MC responsiveness acceptance

Date: 2026-10-07. This milestone addresses the apparent hang of the supplied
ggHH `discrete_mc` command and terminal damage after forced interruption.
Permanent test/gate migration and other examples remain deferred.

## Cause and resulting behavior

The old coordinator waited for a whole wave of native batches before polling
input or status. Default pilot batches contain 4096 points each. For this
artifact and runtime point, the component-wise underflow safeguard sends purely
imaginary coefficients through multiprecision evaluation at nonunit sampling
weights. The workers were computing, while the dashboard and raw-mode Ctrl-C
were not being serviced. A repeated OS signal could then exit through the old
default handler without restoring terminal state.

The caller-owned coordinator now polls during a wave; worker callbacks check
cancellation at point boundaries. Observational worker counts advance while
the native accepted count remains unchanged. Completed batches retain their
original admission order. Incomplete prefixes do not enter means, covariance,
accepted replay state or checkpoints. The terminal owner restores its raw-mode
changes, normal screen and cursor on normal, error, panic-hook and supported
interrupt exits. Drawing and restoration serialize through stderr's native
reentrant lock, so an already-started frame cannot undo final cleanup.

## Scientific and lifecycle controls

The [dispatch review](discrete-mc-dispatch.md) records exact old-versus-new
release comparisons at one and eight workers: mean vectors, full covariance,
diagnostics and accepted checkpoint state match after excluding observational
timings. A [separate native API review](discrete-mc-responsiveness-review.md)
proves missing reservations are reissued with identical task/RNG identities,
completed siblings survive, and the resumed full covariance matches an
uninterrupted run. Pilot cancellation still requires restarting the pilot.

The [terminal review](discrete-mc-terminal.md) covers full and compact PTYs,
normal/error/panic cleanup, raw-mode ownership, ordinary `c`, repeated keyboard
and OS signals, plain/JSON modes and 100 sequential dashboard lifetimes.
Signal callbacks only touch atomics; ordinary Rust performs cleanup. No
terminal mode, listener thread or signal-registration leak was observed.
Only the native macOS signal path was executed. The host's native stack aborts
when attempting panic unwind; pre-abort hook restoration passed, but caught
panic continuation is not claimed.

## Final release command checks

The rebuilt `target/release/fastsecdec` ran the user's ggHH command with
`--full-integral`, the supplied runtime point, `--method discrete_mc`,
32768 points, 32 shifts, seed 20261008 and relative tolerance 0.001. Only worker
count and scratch output destinations varied. Each 120-by-45 PTY run sent an
ordinary `c`, confirmed the job stayed active, then interrupted during the
first pilot wave. The final results were:

| Workers | Interrupt | Time to exit | Evaluations performed | Terminal restored |
| ---: | --- | ---: | ---: | --- |
| 1 | Keyboard Ctrl-C | 0.221 s | 325 | Yes |
| 8 | Keyboard Ctrl-C | 1.118 s | 1362 | Yes |
| 8 | OS SIGINT | 0.992 s | 1505 | Yes |

All three exited normally with the typed `Cancelled` result, zero numerical
failures and no accepted pilot batch. The saved terminal configuration matched
after excluding macOS's transient `PENDIN` flag. Alternate-screen leave and
cursor-show sequences were present. Evidence is the scripts, ANSI captures and
result JSON under ignored `output/integration-hang/`, including
`release-tty-checks.json`.

Separate `--status-json` runs of the same settings confirmed at least three
advancing in-flight snapshots for both worker counts while accepted points
remained zero. They exposed one/eight worker rows respectively, reached 42/189
unfinished points before interruption, emitted no terminal escapes, and
returned the native `Cancelled` status with `pilot_restart_required`. Their
interrupt times were 0.120/0.341 seconds; numerical failures remained zero.
Evidence is `release-status-checks.json` and its JSONL/status captures in the
same ignored directory. These counts are observations, not statistical samples.

The final native release build, scoped strict CLI Clippy, workspace formatting
and whitespace checks passed. This host has Rust 1.99 and native build tools;
`nix-shell` is unavailable. The deferred full-suite migration was not run or
claimed as a gate for this change. No numerical core, dependencies, permanent
tests or other examples changed.

## Limits

This is a responsiveness fix, not a numerical performance or convergence claim.
The conservative precision policy and generated evaluators are unchanged. One
native point or evaluator preparation cannot be cooperatively preempted; the
first interrupt waits for it to return and for result/checkpoint I/O. A second
interrupt restores the terminal and forces exit without saving additional work.
SIGKILL cannot execute cleanup. The original user result and checkpoint paths
were not used by the acceptance probes.
