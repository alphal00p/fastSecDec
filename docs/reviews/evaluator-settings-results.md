# Evaluator settings and dashboard counter acceptance

Date: 2026-10-07. Native Rust 1.99 was used because nix-shell is unavailable
on this host. Permanent test/gate migration and other examples remain deferred.

The final release passed `cargo fmt --all --check`, locked CLI Clippy with
warnings denied, and the locked optimized build. Focused CLI checks passed
default settings, explicit zero/unlimited CPE, direct/tree translation, custom
limits, deterministic-core validation, verbose-output preflight, saved metadata
and readable inspection. One/eight-worker generation of the same control card
produced identical kernel identities. Native library probes additionally checked
reverse completion order, exact IR, runtime parameters and artifact round trips.

Loading the existing ggHH artifact with the new executable produced exactly the
same full estimates, covariance, sector estimates and stopping reasons as the
previous release b7e417f: discrete MC at 2048 points × 8 batches and QMC at 1024
points × 2 shifts, each with one and eight workers, seed 20261008. A historical
checkpoint resumed with zero new evaluations and unchanged complete statistics.

Fresh ggHH generation with four workers and default Horner 10/CPE 1000 completed
in 70.34 seconds, with 30 sectors and observed peak process RSS of 2.6 GiB.
This is an execution observation, not a controlled timing benchmark. The new
artifact is under ignored `output/dashboard-counter-semantics/gghh-optimized.fsd`;
the user's existing artifact and checkpoints were retained.

| Native evaluator statistic, all 30 sectors | Existing zero-Horner artifact | New defaults |
| --- | ---: | ---: |
| Additions | 80,877 | 68,265 |
| Multiplications | 261,859 | 107,954 |
| Exact evaluator bytes | 2,780,572 | 1,590,410 |
| Complete binary artifact bytes | 3,666,802 | 2,477,048 |

Eight-worker runs of the new artifact used the same MC/QMC allocations and seed
as the old-artifact controls. Maximum absolute differences in the full mean were
2.85e-13 (MC) and 1.71e-13 (QMC); complete covariance differed by at most 8.89e-16
and 3.91e-14 respectively. Every sector's means, errors and covariance passed
the declared 1e-8 relative/1e-10 absolute comparison. No numerical failures were
reported. These are floating-point equivalence checks, not bitwise identity
claims across different optimized instruction sequences.

Actual two-round ggHH plain and PTY runs retained the previous completed
allocation, showed explicit pilot/batch waiting states and macOS Free RAM,
and restored terminal modes. Their final means, errors and covariance matched
exactly. Cold JSON inspection and integration of a verbose runtime-parameter
artifact remained valid JSON with `RUST_LOG=info`, confirming quiet loading.

Independent native/portable controls and their limits are recorded in
[the native optimizer review](horner-reuse-review.md),
[implementation evidence](horner-defaults.md),
[CLI review](evaluator-settings-cli-review.md),
[accepted-estimate audit](accepted-estimate-audit.md), and
[memory review](dashboard-counter-semantics.md). Four existing portable controls
and the dedicated settings/IR/runtime-binding/precision probe passed. A separate
portable Gamma control was stopped after 5m43s in native high-precision polygamma
constant mapping; it remains unverified, and the combined suite is not claimed
to pass. No actual Wasm/Pyodide execution is claimed.

The pinned upstream maximum-pair-distance setting remains inactive and is
documented as such. No dependency optimizer was replaced. Temporary probes,
status captures, numerical comparisons and raw timing data remain in ignored
output directories.
