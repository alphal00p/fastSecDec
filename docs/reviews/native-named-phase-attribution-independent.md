# Independent review of named-composition phase tracing

The disconnected draft in `output/probes/named_telemetry` has no mathematical
source blocker. Its explicit diff adds scoped observations around existing
native calls without changing their iteration order, operands, caches, native
series widths, remainder admission or coefficient extraction. The existing
exact naming/restoration identity remains. There is no new differentiation,
series, convolution or evaluator implementation.

Regular native series, coefficient naming, each endpoint axis, each piece's
coordinate products, incremental prefactor-group additions, prefactor series,
final group sums and request lowering have distinguishable spans. Physical
derivative and face cache misses are separately visible; hits keep their
existing behavior. Metadata uses byte sizes, small symbols and integer request
indices, avoiding large expression rendering or restoration merely for a log.

The recorder owns only a thread-local file, counters and a span stack. A scoped
guard resets it on normal return and unwinding; no native Atom or evaluator is
retained globally. Disabled tracing does not evaluate metadata closures. Writes
are flushed, I/O failure is retained for `finish`, and the adapter fails if the
trace cannot be completed. `session_complete` means the recorder closed, not
that the wrapped native `Result` succeeded; that separate outcome must remain
authoritative. An incomplete trace identifies the last observed phase only.

Intervals include logging and metadata overhead and may nest. They must not be
summed as exclusive CPU time or compared as a speedup against the uninstrumented
attempt. In particular, the native result-size metadata and flush belong to the
observed spans. A long call boundary does not identify an internal allocator or
algorithmic cause without further evidence.

The planned ten small controls retain the eight existing composition checks,
then add scoped unwind reset and exact on/off equality of the complete three
Gamma-regulated coefficients, aliases, native width/bound and piece counts.
Those controls, concrete build/input hashes and byte-for-byte restoration of
temporary tracked wiring remain required before the separately bounded actual
attribution stage. Neither compilation nor a completed trace replaces the
independent original-expression oracles and full-order checks required for
scientific acceptance. Source review introduces no execution.

## Executed small gate

The frozen `native-named-telemetry-small-1` control process exited zero in
0.171154873 seconds, with **ten passed, zero failed and two prior replays
ignored**. Both new on/off/reset controls and all eight existing controls
executed. The 21 immutable source, build, executable and dependency entries
pass independent checks before and after execution. There was no timeout;
the retained process report distinguishes its normal child exit from the
watchdog's standard post-exit process-group cleanup. This closes the small
instrumentation gate only. A separately frozen actual attribution stage and
its outcome remain distinct from complete-vector scientific acceptance.

## Actual bounded attribution

The fresh generation-only attempt `native-named-actual-generate-trace-1` passed
independent checks of all 103 frozen inputs before and after execution. Its
unchanged 180-second limit ended in SIGINT/timeout at 180.111590142 seconds,
with 1,736,912 KiB peak resident memory. The 30-GiB cap is separately an address
space limit. Only progress and trace files exist; no complete coefficient roots,
aliases, program or oracle result was produced.

The trace has 6,501 completed spans with valid nesting. Native composition
reached absolute remainder one at relative width seven by 19.567 seconds;
lowering started at 19.735 seconds with 40,723,715 bytes of formal roots over
orders minus six through zero. One completed native constant-face substitution
took 62.343288402 seconds on a 367,174,024-byte mixed partial. A second took
21.424338281 seconds on 125,764,394 bytes. At timeout, the open span stack is
still request lowering, specifically a native face substitution. The independent
span extraction is retained as `independent-span-summary.json`.

This identifies physical derivative-request lowering, especially substitution
after large mixed partials, as the useful next investigation. It does not
establish that coordinate-weight products caused the original timeout, or that
logging times are exclusive CPU costs. The new source-only
`native-interleaved-face-proposal.md` retains native derivatives and substitutions
and requires separate exact small controls before another actual attempt.
