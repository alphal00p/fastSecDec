# Integration dashboard layout and interaction acceptance

This 2026-10-07 follow-up changes integration presentation and terminal input.
Native evaluation, sampling, replay, statistical admission and covariance remain
unchanged. The selected-sector inset consumes already collected operational
metrics; it does not launch evaluator work or increase the observation cadence.

The layout uses native Ratatui panels, tables and gauges. Mouse events use native
Crossterm decoding and cached rendered hit regions. Worker progress is cached
with its numerical observation so mouse/keyboard redraws cannot combine fresh
in-flight counters with an older accepted count. RAM comes from the existing
whole-process/system monitor. No alternate rendering runtime is introduced.

The [independent review](integration-dashboard-polish-review.md) records the
native API reuse, numerical interpretation and focused formatter controls.
Temporary probes and raw terminal recordings remain under ignored `output/`.
The native Rust 1.99 toolchain was used because `nix-shell` is unavailable here.
Other examples and the deferred permanent test/gate migration remain untouched.

## Native presentation and interaction controls

Native Ratatui TestBackend controls passed at 80×24, 80×32, 120×30,
120×42 and 160×48, with a compact fallback also exercised. The 80×24 view keeps
the full sum, two sector rows, selected details, both progress gauges, RAM and
global diagnostics visible. A million/billion-count fixture preserves full
four-significant-digit counts without clipping. QMC coverage remains explicit
for both the full sum and individual sectors.

Independent buffer assertions verified fixed multiplication-dot positions for
real, imaginary and maximum-weight columns across differing uncertainties and
exponents, including −324. Native color/bold styles and `NO_COLOR` behavior
passed. Mouse controls cover ascending/descending headers, compact second-row
headers, selected sector identity, scrolled row hit regions and wheel scope.
Missing values remain last; point sorting uses exact integers, including above
2^53. Overflow is marked explicitly rather than silently truncating a number.
Duration and count probes cover rounding boundaries, unavailable durations,
large/small values and the largest `u64`.

The terminal owner passed eleven focused PTY cases covering ordinary return,
errors, panic cleanup, preexisting raw mode, partial entry failure, mouse input,
plain/JSON exclusion and repeated interrupts. An independent reviewer repeated
the native mouse/forced-exit/raw-mode controls. These are Unix-host results;
Windows terminal execution is not claimed.

## Final release checks

`cargo fmt --all --check`, CLI `cargo clippy --locked ... -- -D warnings`,
and the locked optimized release build passed. The final executable was compared
with the previous validated release (`221469c`) on the existing ggHH artifact
and runtime parameter point, with seed 20261008:

| Method | Work | Workers | Comparison |
| --- | --- | --- | --- |
| Discrete Havana MC | 2048 points × 8 batches | 1 and 8 | Exact equality |
| QMC | 1024 points × 2 shifts | 1 and 8 | Exact equality |

Equality includes the full Laurent estimate, complete covariance, every sector
estimate and stopping reason. No numerical driver, evaluator, sampling or
checkpoint policy changed.

Actual ggHH dashboard runs exercised native mouse header clicks, a selected row,
wheel scrolling, keyboard controls and Ctrl-C. All restored raw mode, mouse
capture, cursor and the alternate screen, returned a cancelled result, and
reported no numerical failures:

| Method | Workers | Terminal | Interrupt cleanup |
| --- | --- | --- | --- |
| Discrete Havana MC | 1 | 140×42 | 70 ms |
| Discrete Havana MC | 8 | 80×24 | 438 ms |
| QMC | 8 | 120×42 | 114 ms |

These bounded smoke runs overlapped the numerical comparisons; the latencies
are responsiveness checks rather than isolated performance benchmarks. Raw
evidence is in ignored `output/dashboard-polish/`: `render-probe.txt`,
`independent-buffer-review.json`, `numerical-equivalence.txt`, `release-pty.json`
and the corresponding native probes and terminal recordings. Permanent tests
and gates were not migrated in this presentation milestone.
