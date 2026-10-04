# Independent boundary CLI and native reporting contract audit

This read-only audit covers the connected boundary CLI, its three real-process tests, the public diagnostic/growth reports and the new sector-contribution reports. It complements the earlier numerical growth and contribution reviews. No scientific computation or HEPKit ownership defect was found. The small public-formatting robustness issue below has been corrected and tested.

## Native ownership and future bridge

The core APIs return owned, serializable Rust structures and typed enums. Physical coefficient values retain the parallel Laurent order and real/imaginary component layout. Boundary pair indices refer to retained raw rows; selected-sector metadata remains present even when no point of that sector was reached. The numerical sampler, native logarithms and native replica statistics remain in their existing owners. There is no new Python dependency, symbolic engine, integration pool, terminal I/O or ANSI formatting in these library modules.

`scan_boundaries` is a bounded convenience loop with an explicit caller-owned `ControlFlow` callback, global point budget and configured retry schedule. The lower-level raw sampler and pure growth analysis remain separately available. Neither API creates worker threads. HEPKit can mirror the typed events and reports without parsing terminal strings or adopting the CLI's interrupt handling. QMC/Havana contribution methods inspect existing session state and do not advance integration, alter grids, promote pilot data or recompute authoritative total covariance by adding shared-sector marginal errors.

The report types are data containers, not a scientific validation certificate or a versioned saved-result envelope. Deserialization alone does not validate provenance or statistical claims. Raw historical boundary rows without physical vectors retain `None`; missing layouts are not inferred. The growth analyzer validates the necessary layout and coordinates before indexing values. The contribution formatter now guards inconsistent public coefficient layouts; its authoritative total and exact offsets remain separate from marginal rows.

## CLI semantics and evidence

The CLI adds only growth tolerance and retry scales, then delegates to the library. `--status-json` serializes the same typed progress events to stderr; final JSON is emitted as one complete document on stdout. Color is confined to the CLI renderer and disabled for plain/nonterminal/`NO_COLOR` output. The library `Display` produces text without terminal control sequences.

Growth flags and incomplete coverage remain diagnostic outcomes. They do not become numerical failures or integrability claims. A numerical evaluation failure remains visible as a raw row with no fabricated value vector, contributes to the diagnostics failure count, and gives a nonzero CLI exit after the complete report is printed. The `ReportedFailure` path avoids appending a second JSON error document. Ctrl-C returns the retained partial report with a typed `Cancelled` execution state; this command deliberately returns normally when cancellation itself is the only event. The display and README explicitly state that acceptable finite growth is not an integrability or threshold certificate.

The three connected CLI tests passed (`output/boundary-cli-tests.log`): flagged-to-acceptable retry history with typed JSON events, a numerical failure producing one JSON report and nonzero exit, and a real SIGINT preserving partial rows and selected-sector metadata. This audit inspected those tests and the implementation without launching another symbolic process during the coordinator's workspace gate.

## Small formatting finding

`diagnostics/growth/display.rs` initially formats a one-based attempt number as `attempt.index + 1`. Native scans cannot produce an index near the machine limit, but the public report is constructible and deserializable. An externally supplied `usize::MAX` index therefore panics in debug formatting and wraps to zero in an optimized build. Use a checked presentation guard or an explicit invalid-attempt marker and cover it with a pure data test. This affects malformed external report presentation, not generated scan results or numerical classification. The finding was sent to the coordinator and report owner without editing frozen source.

Resolved by the report owner with `checked_add` and an explicit invalid-attempt marker. The focused imported-`usize::MAX` data regression passed (`output/boundary-display-import-test.log`). The coordinator's preceding workspace gate passed 220 tests with 10 ignored; the guard has its additional focused passing test. No unresolved finding remains in this audit.
