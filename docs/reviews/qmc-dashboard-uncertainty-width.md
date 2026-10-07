# QMC dashboard uncertainty width

The reported `(σ` fragments were truncations of the missing-error label, not
additional statistical quantities. `QmcSession::live_observation` admits a mean
after one complete shifted lattice and a sampling standard error after at least
two independent complete shifts. A large number of points in the first lattice
does not supply those independent replicas. Missing uncertainty remains `None`;
the formatter does not replace it with zero. Once available, the numerical
uncertainty appears before the exponent in the existing last-digit parentheses.

The CLI split a formatted number into mantissa and exponent cells so that the
multiplication dots align. Its fixed seven-column exponent allocation also
received the trailing missing-error label and clipped it, even when the flexible
mantissa columns had spare space. The same allocation existed in the full-sum
preview and accepted-value panel.

The requested compact label is now `(σ n/a)`. Layout widths use Ratatui's native
`Line::width` on the complete formatted suffix, preserving Unicode terminal
width and row alignment. The table and mouse hit regions share their layout
constraints. The existing compact layout handles crowded auxiliary columns;
an explicit ellipsis replaces a token that cannot fit instead of silently
truncating its exponent or missing-error label. No numerical, integration,
checkpoint or covariance code changes.

Focused render controls use the existing native QMC session and Ratatui
`TestBackend`. They cover the transition after one and two completed shifts,
aligned multiplication dots, clickable headers, compact and wide terminals,
large and subnormal exponents, and dominating uncertainties. The tests exercise
the real formatter and display buffer rather than duplicating layout formulas.
Independent source and execution-evidence review confirmed native API reuse and
unchanged statistical semantics. All three focused render controls pass; the
full CLI binary suite passes 56 tests with one existing ignored test. Strict
CLI Clippy, formatting and diff checks also pass. The terminal widths exercised
are 64, 65, 80, 119, 120, 160 and 240 columns. No expensive ggHH integration rerun
is required for this presentation-only correction.
