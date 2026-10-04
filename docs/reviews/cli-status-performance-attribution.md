# CLI integration status: bounded attribution

This is a diagnostic attribution study, not a new integration result or a claim
of matched whole-application performance. The measurements preceded the narrowly
scoped CLI cadence change documented below; native numerical reductions remain
unchanged.

## Source and fixed-run evidence

The fixed release executable from 561657b invokes `QmcSession::snapshot()` after
every completed worker batch. Only afterward does `Dashboard::integration` apply
its terminal 40 ms or plain 1 s rendering throttle. `--status-json` serializes every
snapshot without that throttle. Worker/package failure handling and cancellation
checks also occur once per batch. A checkpoint is saved on the five-second timer
and at completion.

Native `snapshot` first builds per-sector coverage/time rows and then calls the
native estimator. Democratic estimates request complete shift means from every
Numerica accumulator, compute the shared shift intersection, center each sector
against an anchor and combine the complete vector with Numerica's 106-bit
compensated arithmetic. `complete_shift_ids` is metadata-only, but the estimator
necessarily reduces accepted numerical partials. No alternative estimator or
summation is used in this study.

Both fixed off-shell triple-box diagnostic runs have 1,182 nine-dimensional
kernels, four output coefficients, 1,024 points per shift, eight shifts and two
workers. Each completed 9,456 work packages and emitted 4,728 status lines. The
scalar raw stream is 806,099,007 bytes and rank-two stream 806,655,592 bytes. Recorded
integration-loop elapsed times are 108.146 s and 35.236 s respectively. The full
process watchdog times additionally include artifact loading/compilation and
are 146.205 s and 56.313 s. These observations alone do not attribute the elapsed
time to status work; the scientific runs have different evaluator costs.

Files are retained under `output/diagnostics/triple-box-offshell`, including exact
argv, fixed executable hash, complete checkpoints and saved results. These are
bounded first diagnostic allocations, not independent numerical certification.

## Microprofile boundary

An ignored standalone Rust probe has been prepared under
`output/diagnostics/snapshot-profile`, linked against the same release FastSecDec
rlib and dependency graph. It reads the native saved result, validates its full
scope/manifest, reconstructs the native `IntegrationProblem` metadata and restores
the actual completed `QmcSession` from its native checkpoint. It never loads
`KernelSet`, initializes Symbolica or evaluates an integrand. An empty session
with the same native problem/settings provides a second coverage endpoint.

The measurements separate native estimate, empty/full snapshot, JSON
serialization of a cached snapshot, snapshot plus JSON, native checkpoint
construction, conversion of its bytes to the CLI's JSON value, cached outer
checkpoint serialization, local write/fsync and status write. Warm-up and five
repeats are explicit. Full-state repeated calls must not be multiplied by every
batch and presented as measured whole-run cost: the original allocation passed
through partial coverage, and file/cache/load conditions differ.

## Measurements

The numeric probe completed successfully after the combined scientific/lint gate,
with no other team timing workload. Each operation received one warm-up and five
repeats; raw per-call samples, iteration counts and settings are retained in
`output/diagnostics/snapshot-profile/measurements.jsonl`. The executable linked
against the fixed release dependency graph. Restoring the real scalar/rank-two
sessions took 23.6/22.0 ms, outside repeated measurements.

| Operation (median per call) | Scalar | Rank two |
| --- | ---: | ---: |
| Complete native estimate |1.933ms|1.856ms|
| Complete native snapshot |2.189ms|2.140ms|
| Empty native snapshot |0.286ms|0.239ms|
| Cached snapshot JSON serialization |0.228ms|0.204ms|
| Native snapshot plus JSON |2.344ms|2.356ms|
| Native checkpoint construction |9.200ms|9.431ms|
| Native checkpoint bytes to CLI JSON value |85.222ms|85.000ms|
| Cached outer checkpoint JSON serialization |14.452ms|14.437ms|
| Local checkpoint write and fsync |4.693ms|4.469ms|
| Cached status text write |0.042ms|0.045ms|

Complete native snapshots are 170,842/171,080 bytes; native checkpoint bodies are
5,481,808/5,383,013 bytes. Complete-snapshot repeated-call ranges were 2.155–2.236 ms
and 2.131–2.144 ms. Native estimate reconstruction dominates these snapshot costs;
JSON serialization adds about a tenth as much. The large raw status byte count
therefore does not by itself identify the dominant source of overhead. Parsing
native checkpoint bytes into the CLI JSON value is materially more expensive
than the native checkpoint serializer in this test. That is a distinct five-
second-checkpoint cost, not a reason to alter checkpoint correctness or cadence.

These are empty/complete coverage endpoints. The original runs traversed partial
coverage, and their wall time also includes evaluator/context setup, numerical
work, scheduling, final contributions and storage. Multiplying the full-state
per-call cost by all 4,728 events is not a measured total attribution or a speedup
claim. The local write test includes opening/truncating a temporary file and
cannot represent every consumer's stderr/network behavior.

## Implemented caller cadence

The driver asks the dashboard whether an update is due **before** building the native session
snapshot. Retain existing 40 ms terminal and 1 s plain intervals; use 100 ms by default
for JSON status, with an observational CLI interval option whose zero value
retains per-batch streaming. Force initial, stage-transition, refinement and
final snapshots, including cancellation and failure finalization. This changes
no native estimator or accepted contribution.

Every batch still handles worker/package/submission failures, accepted replay,
evaluation diagnostics, cancellation and periodic checkpointing. There is one
explicit semantic consequence: a numerical-statistics range failure formerly
discovered by the eagerly reduced snapshot would be detected on the bounded
status cadence or unconditional stage/final reduction. It must never be turned
into missing coverage or success. Keeping full estimate reconstruction after
every batch would preserve most of the measured cost, so this detection-latency
tradeoff received coordinator agreement before production edits.

Meaningful regression gates are deterministic cadence/forced-event checks,
existing real Ctrl-C/partial-resume and numerical-failure saved-result processes,
selected-scope stream qualification, and a fixed-artifact native complete-vector
comparison. Checkpoint JSON representation can be considered separately after
profiling; it is outside this proposed cadence change.

The coordinator approved this precise cadence boundary after reviewing the
measurements and detection-latency tradeoff. The implementation now lives solely
in the CLI: `StatusCadence` accepts caller-provided elapsed time, the dashboard
chooses its presentation interval, and the existing QMC/Havana drivers ask before
building a snapshot. `--status-interval-ms` is outside `IntegrationInput` and all
artifact/checkpoint numerical identities. Zero interval preserves per-batch
updates; forced boundary/final events bypass any deadline. No native integration
or checkpoint serializer was changed. Independent source review found no blocker;
the focused executable gate passed all **33 CLI tests** in
`output/status-cadence-cli-tests.log`. This includes deterministic clock/deadline
checks; equal complete estimates, covariance, effective design, checkpoint settings
and replay state at intervals zero and 60 seconds; forced pilot/production/final
events; qualified selected-scope results; and saved numerical failures with
nonzero exit at both intervals. The real Ctrl-C test waits for strictly positive,
incomplete accepted coverage before signalling, then checks cancellation and
resume. Worker elapsed times are deliberately excluded from identity/equality
assertions. Explicit-package formatting and all-target workspace Clippy also
passed (`output/status-cadence-clippy.log`).

## Executed alternating fixed-artifact pairs

The preserved current CLI executable `fastsecdec-eb7e5d8` (SHA-256
`b9bcec8797df82f1a124dc607c2f6a33eb064205257bd80b36ebb07321860851`)
ran the unchanged saved rank-two artifact through its normal validated loader.
The frozen build evidence records dependency/source identities; no artifact
validation was bypassed. Its outer ID is
`80148a88d6cf3bf9b57446c069cea058fa90e775d6b42edc41b0a231706b4519`.
All runs used Kuo38005, 1,024 points, eight shifts, seed 20261004, two workers,
ordinary periodic checkpointing and a final saved result. The only changed option
was `--status-interval-ms`. Three pairs alternated execution order: 0/100,
100/0, 0/100. The team ran no other symbolic computation or heavy build during
these measurements; unrelated host load was not controlled.

| Pair | Per-batch wall | 100 ms wall | Per-batch integration | 100 ms integration |
| --- | ---: | ---: | ---: | ---: |
| 1 | 56.431 s | 49.592 s | 35.339 s | 28.147 s |
| 2 | 56.214 s | 49.030 s | 35.098 s | 28.059 s |
| 3 | 56.485 s | 49.444 s | 35.183 s | 28.135 s |
| Median | **56.431 s** | **49.444 s** | **35.183 s** | **28.135 s** |

Median loading was 20.893/21.014 seconds. Median total wall time decreased by
12.4%, and the integration loop including its ordinary status/checkpoint work
decreased by 20.0%. Median status output decreased from 806,879,392 bytes in
4,730 events to 41,420,645 bytes in 243 events. These are measurements of this
artifact and output destination, not an evaluator throughput claim or a general
percentage for small integrals. The wrapper used here detects completion at
50 ms intervals, so wall times retain up to roughly one final polling interval
plus scheduler latency. That uncertainty is much smaller than the observed
seven-second difference. A separate native wait-based timer is prepared for
future short acceptance cases.

Every run completed all 1,182 kernels and all **9,682,944** planned samples.
After removing only observed worker/load/elapsed times, all six complete JSON
reports and saved results are exactly equal. Native checkpoints differ only in
their per-package worker-cost observations: settings, accepted partials,
coverage, replay state and diagnostics are exactly equal. The comparison includes
every Laurent coefficient, the full covariance matrix, all sector marginals,
exact offsets, effective QMC design, stopping reason and qualified scope. No
variance or scientific term was removed to obtain equality.

Raw argv, stdout, streamed status, checkpoint, saved-result and process records
are retained under `output/diagnostics/status-cadence/pair{1,2,3}-{0,100}.*`.
`paired-summary.json` retains each timing/output count and normalized comparison
hashes. Its report/checkpoint/saved-result hashes are respectively
`655204a87f7d4831d53dcefb11520bead4a5b20ef48321114965b5f5d83c1ef3`,
`261af234f9e84d430a46163d71138fdc8ca0ca833f2b96eb2aeddaeda072b62a`, and
`ae88324ff96bc1399eaaf029c49caea3c3f8cd300d37572221eae520e01ac116`.
The run's sampling uncertainty and independent physical-reference status are
unchanged; this is an observational-overhead comparison with complete numerical
identity, not a new integral-convergence certification.
