# Terminal color and resize acceptance, 2026-10-05

The CLI now shares one color policy between runtime errors, boundary reports
and dashboard styles. `NO_COLOR` keeps the dashboard with terminal-default
colors; `--plain` suppresses the dashboard and report/runtime-error colors.
Clap retains ownership of argument parsing and help. A short/narrow window uses
the existing public snapshot Display instead of squeezing several panels until
their contents disappear. No numerical/status data or integration policy changes.

Actual PTYs were driven through the installed `script` and `stty`, using a
standalone Rust fixture that includes the actual CLI rendering modules and
synthetic public generation/integration snapshots. It initializes no Atom,
kernel or integrator. Source and dependency bindings are frozen under
`output/diagnostics/terminal-dashboard-build-20261005` and the subsequent
`terminal-dashboard-compact-build-20261005`. The latter fixture SHA256 is
`5580f7400a81ee2b40fa294e594df74841429baadbd2e6f0f8bdb4c7d5475520`.
The actual CLI runtime-error controls use frozen binary
`5353d16a8b6400a71ec0eb2eea2239fb5a41e8b0d7f4d4e584e00790360ee84a`.

| Control | Outcome |
| --- | --- |
| Actual CLI missing-result error | Red prefix by default; no color with `NO_COLOR` or `--plain`; expected exit 1 |
| Actual Clap invalid argument with `NO_COLOR` | No color; expected exit 2 |
| Original wide dashboard and plain display | Default color present; plain has no color or alternate screen |
| Updated integration wide → compact → wide | 110×32 → 60×18 → 110×32, exit 0; stage, scope, both coefficients/errors and cancellation hint remain visible |
| Updated generation wide → compact → wide, `NO_COLOR` | Both sizes recorded, exit 0; title, progress, counts and cancellation hint retained without color |
| Updated compact real `q` cancellation, `NO_COLOR` | Positive cancellation observed, exit 0, no color |
| Controlled error after updated compact dashboard creation | Expected exit 1; terminal restored |

Every normal, cancellation and controlled-error dashboard path records identical
actual `stty -g` state before/after, false post-Drop raw-mode status, leaving the
alternate screen and showing the cursor. The final streams were inspected for
the positioned compact content and complete return to wide panels; no claim
about a different terminal emulator or platform follows. Color checks distinguish
explicit foreground/background selection from allowed reset/bold sequences.

The first cancellation probe sent its `q` after the ten-second fixture window
and failed its positive-cancellation assertion. That observation remains in
`dashboard-no-color.stream`; an automated prompt key event then passes, both
before and after compact rendering. The first 60×18 panel display also exposed
the crowding defect; the subsequent compact renderer fixes it. Neither failure
was overwritten or counted as successful evidence.

Raw streams, actual terminal settings, command exits and root's descriptive
`initial-summary.json`/`compact-summary.json` are retained under
`output/diagnostics/terminal-policy-20261005`. Root independently extracted the
combined workspace gate: **305 passed, 0 failed, 18 ignored across 57 summaries**.
Formatting and workspace all-target Clippy also pass. The independent source
and outcome review is in [terminal-policy-independent.md](terminal-policy-independent.md).
