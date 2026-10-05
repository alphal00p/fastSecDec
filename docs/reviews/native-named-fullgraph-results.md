# Native original on-shell fullgraph attempts

No complete original on-shell fullgraph artifact or integral estimate is yet
accepted. The public captured representative is independently accepted; its
scope is recorded separately in
[native-named-public-actual-results.md](native-named-public-actual-results.md).

The ten-parameter `native-named-fullgraph-2` trial was intentionally cancelled
by the coordinator at 372 of 1,026 representatives after 1,006.899916801 seconds.
Peak RSS was 14,077,364 KiB. It returned the typed `generation cancelled` error,
exited 1, was reaped, and passed all 58 immutable hash checks. It produced no
artifact or dependent inspection/integration. This was a manual design decision
to use existing exact native family preparation, not a timeout. The retained
cancellation and independent outcome are linked from the
[protocol](native-named-fullgraph-protocol.md).

The preceding `native-named-fullgraph-1` release build succeeded, but its
freezer rejected a logging-only feature difference. That failed freeze remains
separate. Independent review accepted the narrow explicit release-feature
comparison used by the second attempt; no scientific feature changed.

The prepared route uses committed CLI milestone `e42a017`, with the exact same
original graph, kinematics, order-zero request and production SymJIT O2. It adds
only `NativeNamed` and `SingleUnitTerm { max_states: 32 }`. Its fresh evidence is
under `output/diagnostics/native-named-prepared-fullgraph-*`:

| Attempt | Outcome |
| --- | --- |
| 1 | Pre-build metadata check rejected a new documentation-only Markdown file. No Cargo or scientific process started. The narrowly permitted documentation paths are now recorded. |
| 2 | Pre-build source overlay used absolute keys against the prior relative-key map and failed its hash check. No Cargo or scientific process started. Exact same-path replacement fixed the metadata mismatch. |
| 3 | Release build and independent preflight passed. All 1,026 coefficient expansions and sector compilations completed, but allocation failed before artifact publication. The whole process was reaped after watchdog cleanup of its core dump; no downstream stage ran. |

Both failed metadata attempts remain preserved. The third binary SHA-256 is
`033b8ed667c220f0c6b7934f654ed87b60bb4ddadd684bb3ddbf37b6edc76919`;
its manifest SHA-256 is
`d28da833afe38c2b346447f50ad8aec26e122d814b2af2d4e7d6b53d44e6f258`.
The accepted preflight SHA-256 is
`b59fd4c7d2aef267c7504c89e8be5adf4200b981258c58348bcc02ffeb490829`;
it verifies 227 accepted source and 1,414 native source bindings.
The third attempt binds the
accepted CLI source overlay and unchanged production/native sources. The
prepared native runtime will retain its actual family report and active
coordinates through ordinary artifact provenance. It uses the unchanged
1,800-second generation, 180-second cold inspection and 180-second complete
1,024-by-eight integration limits, stopping on failure. External reference
generation and native generation remain serialized; assigned release compilation
may overlap and is recorded separately. No performance or convergence claim is
made by build completion or the fixed allocation.

The third trial completed all coefficient expansions at approximately
1,532.99 seconds, with 1,531.885429 seconds in the exclusive named coefficient
phase. Its last sector-compilation callback was at 1,668.039795 seconds,
reporting 1,026 of 1,026 sectors and 127.529335 seconds of compilation. The CLI
reported eight active parameters. These observations establish completed
coefficient generation and sector compilation, not a completed portable
artifact or an independently compared integral.

After that callback, stderr reported `memory allocation of 8388608 bytes failed`.
The first passive failure observation was around 1,713 seconds, with RSS
26,574,292 KiB and virtual size 31,456,076 KiB, close to the 30-GiB address-space
cap. The process entered `CoreDumping: 1` before the deadline. The callback
precedes native artifact initialization and the CLI wrapper's construction and
save, so existing status cannot distinguish the exact failing allocation within
that persistence interval. Earlier monitoring records called it a save-stage
failure; this broader location is the source-backed interpretation.

The watchdog sent SIGINT at 1,800.003 seconds and SIGKILL at 1,805.003 seconds;
the final timer reports **1,807.811116 seconds**, signal 9 and `timed_out: true`.
That timeout followed the allocation failure while the process was dumping
core. Peak RSS was **26,771,064 KiB**; user/system CPU were
1,444.243597/244.988379 seconds. The coordinator subsequently authorized stopping
the core dump, but the ownership guard found it already absent, so no manual
signal was sent. The wrapper was reaped with exit 124 and all recorded process
groups were absent. All 60 frozen hashes still pass.

No artifact, cold inspection, integration, checkpoint or result was produced.
The attempt retains `generate/report.json`, `allocation-failure-observation.json`
and `terminal-slot-release.json`, with the source-location clarification in a
separate record. The independent terminal audit accepts the failed-attempt
record, including all 1,026 coefficient-completion events, process cleanup and
60 immutable checks. Its `independent-terminal-review.json` SHA-256 is
`552de8975a2b527df25034ab71f9f0f53a1a5f727821de47b5d86276ea889307`.
See [native-persistence-memory-audit.md](native-persistence-memory-audit.md) for
the source-backed persistence findings. No unchanged-input retry or larger
resource limit is authorized by this outcome.
