# Phase B CLI, pilot and checkpoint audit

Date: 2026-10-09. Independent read-only review of the in-flight fixed-contour
implementation. This review owns this document, not the implementation files.
It is not a Phase B completion claim. Final provenance presentation was already
known to be unfinished when this slice began.

## Findings requiring resolution

1. **Serial preflight visits mandatory empty exact records.**
   `isolated.rs` currently loads every `sector == None` record and binds it with
   the requested `always`/`pilot` policy. `IndexedWriter::write_unit` always
   emits an exact record before a numerical sector. For a stochastic chart,
   `binary::partition(None)` stores zero exact coefficients and no chart
   metadata; the strength schema still marks it contour-capable. The current
   certified `ContourBinding::new` requires metadata for enabled validation,
   so this path rejects the empty record before sampling starts. The aggregate
   `Off` binding later in setup does not fix that earlier per-record binding.

   The existing bubble artifact was independently inspected with an ignored
   direct-Rust probe: record zero has `source_indices=[]`, no metadata, and
   contour capability. The available 17:45 compiled library still accepted its
   binding and predates the stricter checker, so the predicted failure against
   the current source must be confirmed with the coordinator's rebuild.
   Assembly folds entire charts, rather than splitting mixed stochastic/exact
   coefficients; there is no evidence here of a lost nonzero mixed-chart offset.
   Skip only a provably empty record or preserve its explicit no-work admission;
   do not weaken metadata requirements for actual contour contributions.

2. **Ordinary selected-sector integration pilots excluded contributions.**
   `main.rs` runs `contour_pilot::run` over every validation chart before
   canonicalizing the requested result scope. A selected good sector can thus
   be refused because an excluded stochastic or exact chart has an invalid or
   unresolved contour. Serial stochastic pilots follow scheduled sectors and
   its exact setup skips checks under `ExcludeAll`. Pilot ownership/admission
   should follow the selected mathematical contributions consistently. Add a
   regression where an excluded chart fails validation while the selected chart
   is admissible; this should succeed without turning checks off for the latter.

3. **Native `always` preflight remains caller-enforced.**
   `SectorValidation::Always` checks production arguments but does not require
   the configured pilot to have finished. The CLI does perform a pilot before
   production. Native tests currently sample after fewer than the configured
   pilot points. Decide and document the native contract, or enforce the same
   readiness gate used for `pilot`, to make the approved "pilot plus production"
   policy consistent for future HEPKit callers. This finding does not claim
   that `always` silently skips its production sign checks.

All findings were sent to their implementation owners promptly. Resolution
and rebuilt regression evidence must be recorded before accepting this slice.

## Verified design boundaries

- `ContourArgs` changes only runtime settings and rejects nonfinite/nonpositive
  fixed strength. Any finite positive strength has no artificial upper bound.
  Pure-Serde generation overrides avoid constructing native graph/algebra
  objects in the serial coordinator. Dynamic mode is explicitly unavailable at
  this fixed-mode milestone.
- The pilot creates a separate native `MonteCarloRng`, domain-separated by
  protocol, bound kernel identity, master seed and chart. It uses Numerica's
  uniform continuous grid, explicitly includes the cube centre, and evaluates
  retained subtraction-face arguments through the native checker. It neither
  reserves numerical session work nor submits observations. Repeated validation
  coordinates after eviction are intentional diagnostics, not duplicate
  production evidence. Hash domain separation is not a claim that finite random
  coordinates can never coincide by chance.
- Every enabled CLI path re-runs a pilot after rebinding. Changed kinematics or
  strength change the bound identity; no old pilot certificate is reused by the
  CLI. Native policy-only changes retain valid pilot evidence, whereas rebinding
  clears readiness. The pilot is finite sampled evidence, not a global proof.
- Ordinary checkpoint identity removes only the contour `validation` field;
  physical parameters and mathematical deformation stay in the identity.
  Serial identity inherits this rule and additionally permits residence-time
  changes. Validation settings do not enter production RNG/epoch allocation.
  The strength and parameter changes therefore remain incompatible with an old
  numerical checkpoint, while checked-to-unchecked continuation is permitted.
- Serial workers bind and pilot a selected native record before constructing
  its weighted context. The initially reserved numerical allocation remains
  unchanged while the pilot runs. Failure/cancellation reaps the process before
  releasing its reservation. Existing run/lease and replica identities fence
  pilot, load and numerical completion events.
- A retained worker hashes parameters, precision/replay settings, contour
  settings and vector shape; it refuses a different residency identity. A
  continuation uses the existing context and replay frontier. Worker number,
  process restart and validation RNG do not reseed production work.
- Heavy records and numerical returns travel by file. Control frames have a
  one-MiB limit; parent and child channels have capacities two and one. Progress
  is consumed as snapshots. The setup worker is reaped before serial production
  starts, and sector replacement reaps the prior process before admitting its
  successor. No new library-owned pool or alternate RNG was introduced.
- Metadata-free aggregate exact binding uses `Off` only after the intended
  per-record checks. It avoids rebuilding contour maps or retaining all rich
  chart metadata merely to add exact offsets; finding 1 must be addressed for
  the per-record stage to work with the current checker.

## Remaining evidence and reporting

The coordinator is rebuilding with the independently tested Symbolica ball
adapter and latest SymJIT callback-lane fix. This review did not compete for the
shared Cargo build slot. The independent probe source is ignored at
`target/contour-exact-record-probe.rs`; final accepted tests should live in the
normal regression suite.

Still required: actual ordinary/serial checked-pilot runs, policy-only resume
with identical accepted production streams and statistics, failure/cancellation
during resident preparation, and selected-scope regressions. Pilot provenance
and numerical check counters must survive into final reports/checkpoints rather
than only the transient dashboard. Resident contexts accumulate check counters
across tasks, so aggregation must use per-allocation deltas or replace a known
resident cumulative snapshot; summing cumulative values would double count.

The checker certifies symbolic polynomial-map signs at the supplied floating
inputs using native exact-rational ball mapping. It does not certify every
rounded intermediate in the separate compiled f64 kernel. Reporting must retain
that distinction, together with whether production actually received checks.

## Resolution review

The runtime owner has addressed all three initial findings in current source:

- Metadata-free indexed exact records are exempted only when there are no
  stochastic sectors and every exact expression is the literal zero. Nonzero
  contributions without provenance still fail. A dedicated native regression
  exercises both `always` and `pilot`.
- The CLI now canonicalizes ordinary result scope before presampling and filters
  stochastic/exact charts accordingly. Native per-chart readiness prevents a
  scoped pilot from unlocking an excluded sector. The native regression covers
  an excluded bad chart as well as the selected good chart.
- `always` now requires preflight readiness before running its production checks.
  Both policies retain readiness across worker clones of the same bound owner,
  while separately cloned kernel owners require their own valid pilot.

These changes were source-reviewed; the coordinator owns the rebuild and full
CLI execution evidence. Additional small CLI tests should exercise serial
zero-offset setup end-to-end, selected-scope pilot filtering, cancellation while
a worker is preparing, and checked-to-unchecked checkpoint continuation.

The reporting slice now provides native `ContourPilotProvenance` and
`ContourRunReport`, plus optional checkpoint-compatible contour diagnostics.
Ordinary QMC, per-sector MC and discrete MC drain worker counters after each
batch, preserving distinct production and adaptation totals/precision maxima.
A focused real-kernel regression checks consecutive batches on one resident
context and identical values across `always`, `pilot` and `off`. The runtime
owner independently reviewed stage capture and atomic counter merging. Final
root-owned collection of serial pilot records and report attachment is in flight.
Recorded counters deliberately mean available recorded history; merging a
legacy checkpoint without contour counters does not certify its missing history.

Durable pilot provenance is now implemented independently of statistical
compatibility. Ordinary and serial checkpoints store the latest validation
options and a bounded list of completed pilot records, deduplicated by bound
kernel identity. Restoring an `always` checkpoint under `off` retains that earlier
pilot evidence while reporting the current production policy accurately. Fresh
pilots replace the same owner's record; they do not increase the record count
with residence visits. Stored proof records do not unlock native readiness or
authorize skipping a new binding's preflight. Ordinary QMC, per-sector Havana and
discrete Havana save this evidence at their existing checkpoint boundaries.

The serial coordinator's source was independently reviewed after this change:
restored pilot records seed its owner map, new exact/preparation records merge
into it, and final reports/checkpoints receive the same bounded evidence. Pilot
events remain fenced by the current reservation identity. Per-batch native
counter drains prevent resident reuse from counting prior checks again. A
numerical failure now publishes its successful check prefix through an
operational progress event before returning the failure; unfinished numerical
work still cannot enter accepted integration statistics.

The native regression verifies proof serialization, unchecked continuation,
deterministic owner ordering and repeated-residence boundedness. The coordinator
owns end-to-end tests covering the normal/serial generation and integration
matrix, all three validation policies, and policy-only checkpoint continuation;
the four CLI subprocess tests passed on 2026-10-09:

- `fixed_contour_universal_artifacts_policies_and_resume_preserve_sampling`:
  symbolic/numerical-dual generation, normal/serial generation and integration,
  all three validation policies, analytic complex bubble coefficients, identical
  production values across checking policies, and checked-to-unchecked resume.
- `run_forwards_fixed_contour_to_serial_subtraction_and_exact_offsets`:
  CLI forwarding, endpoint subtraction and a nonzero Laurent pole/finite part.
- `selected_sector_preflight_excludes_an_unresolved_chart`: exactly one regular
  selected chart succeeds while the stationary-zero chart is refused, for both
  ordinary and serial integration. Only selected-chart proof is recorded.
- `cancellation_during_preflight_admits_no_production_work_and_reaps_residents`:
  actual incomplete pilot progress is observed before SIGINT; ordinary and
  serial execution admit no production points and serial workers are reaped.

The first pair ran in 7.75 seconds and the second pair in 1.27 seconds in the
debug test build with the two locally tested owner fixes. These are test-suite
durations, not production performance claims. Both native status regressions
also passed, including checkpoint evidence preservation/boundedness and distinct
adaptation/production counters. The CLI contour unit filter passed seven tests,
including the real-kernel resident counter-drain regression.
The runtime agent independently reviewed the checkpoint/provenance changes and
found no blocker. In particular, latest-owner deduplication does not union
separately scoped pilot evidence, and ordinary checkpoint scope compatibility
prevents an unrelated scope from inheriting numerical statistics.

The public diagnostic commands now use the same physical/contour binding and
explicit all-chart preflight before sampling. Their additive JSON diagnostics
drain the actual native evaluator owners. Benchmark counts explicitly include
warmup and measurement calls; boundary scans label their probes separately.
Neither creates accepted integration observations. A fifth subprocess control
passed in 0.52 seconds across `always`, `pilot` and `off`, verifying fixed-strength
overrides, pilot provenance, the exact warmup/measurement check count, unchanged
native boundary-report decoding and an undeformed runtime-parameter control.
The runtime agent independently reviewed these evaluator/counter ownership paths
without finding a blocker.
