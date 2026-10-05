# Real terminal acceptance of CLI geometry dispatch

The coordinator exercised frozen debug binary SHA-256
`436e79552448a1f417fa31c8582757f9d688981936a88cfee62192a10a9a5bc0`
from `output/diagnostics/cli-geometry-dispatch-build-1`. Source, test and binary
hash checks passed before and after the tests. These are terminal correctness
checks, not generation or parallel-speed measurements.

The first ignored driver used GNU `timeout` without `--foreground` inside
`script`'s PTY. It produced no dashboard output or geometry observation and
ended at the external 45-second deadline plus five-second kill grace. Its
exit 137, stream and original driver remain in
`output/diagnostics/cli-geometry-dispatch-pty-1`. Local `timeout --help` documents
that `--foreground` is needed for the command to read the TTY and receive TTY
signals. This was an orchestration failure; no CLI scientific result or terminal
acceptance is inferred from that attempt.

The fresh second attempt changes that driver setting only and uses the same
binary. It runs the actual `four_loop_hard.toml` card with two geometry workers
on CPUs 8–10. After observing parallel geometry, the driver resizes from 110×32
to 60×18 and back, then sends `q`. The raw stream records both compact and full
generation views, chart work and subsequent cone work, and finally
`sector decomposition cancelled`. The command exits 1; no artifact exists.
Terminal settings match byte-for-byte before and after execution; the stream
contains alternate-screen exit and cursor restoration. With explicit `NO_COLOR`,
there are no foreground color sequences. Evidence is retained in
`output/diagnostics/cli-geometry-dispatch-pty-2`.

Two additional real-PTY commands generate the triangle with two geometry
workers. Both exit 0, write artifacts, restore identical terminal settings and
leave the alternate screen with the cursor visible. The first retains the
environment's monochrome policy. The second explicitly removes `NO_COLOR` and
records ten foreground color sequences. Their evidence is in the sibling
`cli-geometry-dispatch-pty-normal-1` and `cli-geometry-dispatch-pty-color-1`
directories. All commands and owned sessions are reaped; no native process is
left running. Existing source/library and independent numerical gates retain
their own scopes.
