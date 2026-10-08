# Streaming generation implementation notes

Implementation evidence for the generation slice of `SERIAL_MODE_PLAN.md`.
This records the author's checks; the separate scientific, memory and ecosystem
reviews remain required before accepting the complete serial-mode milestone.

## Native reuse and boundaries

- `generation::streaming` accepts the existing `ParametricIntegrand`, generation
  options, native runtime symbols and `RuntimeMassConstraint` objects. HEPKit's
  graph import, scalar contraction and parametrization remain in their existing
  owners. No graph or expression parser is introduced.
- Source preparation calls the existing `Geometry::compute`. The preparation
  child spools its native maps, source context and target coordinate symbols.
  Geometry's complete collection of primitive maps remains necessary shared
  preparation overhead, rather than a collection of mapped sector expressions.
- Chart construction calls the ordinary symbolic mapper or numerical-dual
  discovery API. Exact symmetry calls the existing Graphica canonical graph
  comparison and verified native parameter permutation. A compact bucket hash
  locates candidates; it is never accepted as proof of equivalence.
- Formula construction calls the existing numerical-dual key and recipe
  builder. A persisted formula includes a native signature; workers reconstruct
  and compare the exact key before accepting a reused recipe. The existing
  symbolic fallback remains available for regulator poles.
- Native record transport uses Symbolica `State::export_partial`, `StateMap`,
  and native `Atom`/`Symbol` codecs, checked in the pinned owner's source and
  exercised by the Rust round-trip/scientific tests. No string parsing or
  expression expansion is used to restore a source, chart or subtraction recipe.
- Sector workers call the existing subtraction, Laurent assembly, compilation
  dispatcher and indexed artifact writer. The library creates no process pool,
  thread pool or hidden integration/generation loop.

`State::export_partial` also exports current polynomial-variable lists and finite
field registrations. `State::reset` does not reclaim its append-only offsets and
would invalidate live atoms, so the implementation does not call it. The CLI
uses native process exit to reclaim Symbolica caches and allocator high-water
memory. A worker may hold its source context, one active representative and the
rich metadata belonging to that representative's symmetry class. Exact
symmetry admission holds one source and one candidate density at a time.

## Persistence and recovery

The coordinator handles only JSON descriptors, exact byte digests, ordered
completion receipts and bounded byte copies. It never imports a native record
or restores a sector evaluator. Each native record is independently decodable;
each stochastic unit has a local Laurent layout and source-chart mapping. The
indexed catalogue performs global output projection without recompilation.

The journal registers a request before issuing it. Native files are flushed and
synced before the corresponding JSON receipt is atomically published. A resume
validates scientific input/dependency identity, persisted request identity and
native record digests; it can adopt an unacknowledged but durable completion.
Worker count and the `generation.serial` choice do not alter scientific identity.
Incomplete staging is never accepted as a complete artifact. The common
artifact publisher makes the immutable data file durable before replacing the
manifest. Successful publication precedes journal completion and staging cleanup.

The coordinator holds a stable OS lock file throughout a run; every native child
inherits the same locked open-file description. A crashed coordinator cannot
release residency while its children survive. Native control traffic uses the
CLI's separately framed socket, while Symbolica/native stdout goes to worker
logs, so banner or diagnostic output cannot corrupt IPC.

The fused sector interval covers Laurent construction, evaluator preparation,
compilation and durable sector output. Its caller wall time is recorded once in
`coefficient_expansion_seconds`; it is not added again as overlapping aggregate
compilation wall time. Per-sector receipts retain individual compilation times.

## Validation recorded so far

`cargo test -p fastsecdec --lib generation::streaming::tests --locked --
--test-threads=1`: **7 passed** on 2026-10-08. These cover:

- Complete complex Laurent-vector parity for symbolic/numerical-dual generation
  and Taylor/IBP subtraction, including frozen runtime mass context.
- Exact symmetry multiplicities and all original source-chart metadata.
- Shared formula reuse and regulator-pole symbolic fallback.
- Exact-only and empty inputs; unregulated divergences remain errors.
- Corrupt, foreign and incomplete records, and cancellation before receipt
  publication.

Both process-level CLI tests passed on 2026-10-08. They cover normal/serial
mathematical identity, independent native restoration, changed-worker-count
resume, and actual coordinator SIGKILL followed by inherited-lock release and
successful recovery.

The existing ggHH double-box scientific input completed native serial generation
and both integration methods on 2026-10-08. The ignored validation card retains
the original graph, numerator, masses, runtime invariants and finite epsilon
order; only its backend selects supported numerical-dual generation with Taylor
subtraction. Two recyclable workers produced all 30 sectors and four subtraction
formulas, with 26 formula reuses. The final indexed artifact has 31 MiB of native
data and a 121 KiB manifest. Successful generation removed its heavy staging
directory and retained the completed journal and stable lock.

These bounded debug-build measurements validate execution and residency, not
release performance or a convergence target:

| Work | Wall seconds | Peak tree RSS | Peak coordinator RSS | Maximum native children |
| --- | ---: | ---: | ---: | ---: |
| Generation through finite order | 309.21 | 492.5 MiB | 25.0 MiB | 2 |
| QMC, 2 shifts of 1024 points per sector | 13.09 | 153.8 MiB | 42.8 MiB | 2 |
| Havana MC, 2 batches of 1024 points per sector | 26.37 | 181.7 MiB | 69.0 MiB | 2 |

Both integration runs completed 61,440 evaluations and the complete complex
Laurent vector, stopping honestly at the explicit work limit. Their finite
coefficients are `i*(355.1494 +/- 1.6991)` and `i*(358.3537 +/- 3.8700)`, compatible
at 0.76 combined standard errors; neither claims per-mil convergence. The
generation and integration artifacts/logs remain ignored validation outputs.

The final release binary also passed the same QMC smoke against this artifact:
all 30 sectors and 61,440 evaluations completed in 3.29 integration seconds
(3.77 seconds including monitored startup/shutdown), using at most two children,
58.8 MiB aggregate RSS and a 10.3 MiB coordinator. Its finite coefficient was
`i*(356.1969 +/- 1.6942)`, compatible with the debug QMC and Havana MC results.
It likewise stopped at the explicit work limit. This confirms final-binary
execution; it is not a convergence benchmark. The release command substitutes
`target/release/fastsecdec` and the new `gghh-release-qmc.checkpoint.json` path in
the QMC command below.

The concrete commands below used a hardlink to the debug CLI so concurrent test
builds could not replace a live worker executable. The ignored `gghh-dual.toml`
is copied from `examples/gghh_double_box/run.toml`, adds
`generation.mode = "numerical_dual"`, and adjusts its three relative input file
paths back to that same example directory. No tracked example was changed.
Each command ran inside an owned-process monitor with a 600-second wall limit
and 15 GiB aggregate RSS limit; the monitor stopped only its own process group.

```sh
output/serial-validation/fastsecdec-gghh-probe-v2 --plain --json --status-json \
  generate output/serial-validation/gghh-dual.toml --serial --workers 2 \
  --output output/serial-validation/gghh-dual-v2-serial.fsd

output/serial-validation/fastsecdec-gghh-probe-v2 --plain --json --status-json \
  integrate output/serial-validation/gghh-dual-v2-serial.fsd \
  --parameters examples/gghh_double_box/point.toml --serial 0.01 --workers 2 \
  --method qmc --points 1024 --shifts 2 --max-rounds 1 --target-order 0 \
  --checkpoint output/serial-validation/gghh-v2-qmc.checkpoint.json

output/serial-validation/fastsecdec-gghh-probe-v2 --plain --json --status-json \
  integrate output/serial-validation/gghh-dual-v2-serial.fsd \
  --parameters examples/gghh_double_box/point.toml --serial 0.01 --workers 2 \
  --method mc --points 1024 --shifts 2 --max-rounds 1 --target-order 0 \
  --checkpoint output/serial-validation/gghh-v2-mc.checkpoint.json
```

The first bounded attempt exposed a transport bottleneck: formula workers were
sleeping with roughly 3.5 MB queued in each control socket while repeated native
cancellation polls produced identical status snapshots. The completed run above
includes the fix below; its four formulas finished in 6.44 seconds and control
queues remained empty. Increasing-sector RSS tests remain a separate milestone
acceptance check.

Native formula callbacks double as cancellation polls and can occur much more
often than a dashboard refresh. The generation worker therefore coalesces
same-stage progress snapshots to at most 20 updates per second before IPC,
while immediately sending stage transitions. The separate coordinator watchdog
still detects parent death independently. This prevents a bounded status queue
from retaining native work behind repeated identical progress messages.
