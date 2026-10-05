# Independent terminal color-policy review

Source and retained PTY outcome review, 2026-10-05. Inspected the shared `ColorPolicy`, its callers in
the dashboard, boundary display and main runtime error path, and the existing
terminal construction/Drop behavior. No blocker was found in the changed
behavior. Executed PTY evidence is reviewed below.

Every explicit dashboard foreground style now goes through the same policy.
`NO_COLOR` suppresses those colors while retaining the live dashboard, layout,
borders and bold text. `--plain` still disables the dashboard and now disables
the main runtime error color. Nonterminal report/error streams remain plain;
JSON branches retain their original behavior. Boundary rendering uses the
same decision rather than a duplicate condition.

Clap handles argument parsing before this runtime policy is captured. Its help
and parse-error presentation therefore remains governed by Clap; the README
correctly limits the `--plain` claim to reports and runtime errors. This review
does not claim new parser-color behavior.

The existing dashboard Drop restores raw mode, alternate screen and cursor
on normal return and on an error after construction. An actual PTY remains
necessary to establish that restoration and resizing behavior. The ignored
fixture `output/probes/terminal_dashboard.rs` includes the actual CLI dashboard
modules and synthetic public generation/integration snapshots. It performs no
Atom, evaluator or integration work. Its normal/error/cancel modes retain
post-Drop raw-mode status; the cancel mode fails unless a real key event is
observed, and repeated terminal-size records support a resize control. The
fixture requires a fresh readiness file and a real stderr PTY.

The fixture itself does not call main's error renderer. Runtime error colors
must additionally be checked with the current CLI and an ordinary missing-file
error. PTY acceptance should retain raw streams and external terminal settings
before/after, rather than relying solely on the fixture's raw-mode query.

## Executed PTY evidence

Reviewed the coordinator's retained raw streams, summaries, exit codes and
external terminal settings under `output/diagnostics/terminal-policy-20261005`.
Independently compared every recorded `termios.before`/`termios.after` pair
for the four compact cases and the error/color/cancellation/plain controls;
all compared equal. Raw stream inspection confirms alternate-screen entry
and exit plus cursor restoration for the dashboards, and no alternate-screen
entry in plain mode. The fixture's post-Drop raw-mode reports are false.

Default dashboard and runtime errors contain foreground color sequences.
The NO_COLOR dashboard, runtime-error and Clap parse-error streams contain
none; the plain dashboard and runtime error are also uncolored. This was
independently checked on the raw bytes, without requiring identical counts
of redundant style/reset escapes from terminal redraws.

The first cancellation control is preserved as an exit-one failure: its key
arrived after the ten-second diagnostic window. An automatically timed key
control then passes and records actual cancellation. Both paths restore the
terminal. The original fixed-panel view also exposed crowding at 60x18; that
observation is retained and led to the reviewed compact path using existing
public snapshot Display, without changing numerical data or callbacks.

The second frozen fixture, SHA-256
`5580f7400a81ee2b40fa294e594df74841429baadbd2e6f0f8bdb4c7d5475520`,
exercises both generation and integration through 110x32 to 60x18 and back.
The four compact controls pass: normal generation/integration exit zero,
controlled error exits one, and real-key cancellation exits zero with
`cancelled=true`. Normal/cancellation summaries retain both observed terminal
dimensions; the controlled-error stream retains its compact redraw. The
coordinator inspected the displayed headers, coefficients and controls.
The independent source read confirms the narrow view uses the same public
status plus stage/scope/time rather than a reconstructed result. The complete
frozen fixture manifest was independently rechecked.

This accepts the tested terminal behavior, not all possible terminal emulators
or arbitrarily tiny windows. It creates no scientific or performance result.
The same final production source also passed the 305-test workspace gate,
formatting and all-target Clippy recorded with the geometry-context milestone.
