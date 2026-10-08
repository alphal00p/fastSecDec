# Independent indexed-artifact and memory review

Independent review, 2026-10-08. The artifact and streaming implementation
owners did not perform this review. Source ownership and bounded analytic
residency checks below pass after the listed corrections. The complete serial
mode acceptance still depends on the separate statistical/stream audit and
the bounded ggHH completion. No Cargo build was run concurrently with the
coordinator's validation build.

## Scope and positive findings

Reviewed `SERIAL_MODE_PLAN.md`, the current first-phase requirements, native
`kernel::indexed` and output projection, `generation::streaming`, CLI indexed
publication, resumable generation orchestration, process ownership and the
serial integration load boundary.

The native APIs are synchronous, caller driven and free of process/thread pools.
The CLI coordinator handles ordinary descriptors and disk paths. It neither
queues generated evaluator owners nor restores completed sectors to assemble
an artifact. Source, mapped charts and shared formula records are spooled;
sector jobs restore their active representative and associated inspection
metadata. Generation process slots are released only after an OS wait. Native
stdout is separated from framed control messages, whose size and queue length
are bounded. The inherited stable lock prevents a replacement coordinator from
acquiring ownership while a previous worker remains alive.

Native symmetry lookup hashes are only candidate buckets. Admission compares
Symbolica canonical graphs and verifies the exact density permutation. Reused
subtraction formulas reconstruct and compare the native full requirement key.
The serialization envelopes retain Symbolica Atoms, aliases and symbol context;
they do not introduce a parser, alternate algebra system or graph canonicalizer.
HEPKit input loading remains the existing native path. No additional scalar
master-integral or reduction implementation was introduced.

The indexed format separates local coefficient layouts from the complete
catalogue. Numerical loading scatters into the global real/imaginary Laurent
vector without calling an evaluator composer or optimizer. The original native
program bytes and local precision-rescue state remain intact. Exact offsets are
separate records, and numerical records contain zero exact offsets. Catalogue
identity excludes completion order and physical offsets; the CLI separately
binds scientific input and saved representation identities. Checkpoints must
continue to bind both identities and the physical point.

The data file is immutable and unique to a publication. Its contents and directory
entry are synced before an atomic manifest replacement; the manifest is the
commit point. Failed publication leaves the previous manifest's data untouched.
`IndexedWriter` checks input receipt digests and poisons itself after a failed
append. Required bounds/layout/backend checks remain separate from optional
expensive certification. Legacy ordinary reading is retained.

## Findings and required follow-up

1. **Exact setup retained unrelated chart metadata.** The initial
   `IndexedReader::load_with_progress(true)` accumulated every exact-only
   record's rich charts in a temporary vector, only discarding them at the end.
   Analytically integrated charts may retain substantial pre-subtraction data.
   The coordinator fixed the exact-only branch to immediately discard
   these charts; the correction was independently re-read. That change alone
   does not resolve the native context issue below.

2. **Native context import can retain historical polynomial resources.** In
   locked Symbolica revision `1deccb8`, `State::export_partial` narrows symbol
   selection but exports all process-global finite-field and polynomial-variable
   tables. Polynomial variables may own function or power Atoms. `State::import`
   inserts those tables into append-only global storage. Therefore dropping a
   record, metadata or its `StateMap` cannot promise to release that storage.
   Exact setup in the long-lived coordinator can accumulate resources from
   completed sectors. The coordinator moved this setup to a recyclable child
   that returns a compact numerical result manifest. Independent source review
   of `isolated.rs` confirms that it is reaped before sampling begins. The child
   also handles ordinary generation when the following integration is serial,
   releasing generation allocator and native-state high-water marks first.
   The exact-setup child itself can still accumulate those resource tables
   while walking all exact records; process recycling does not bound that
   transient peak independently of sector count.

   Public API/source/probe evidence: `state.rs` exposes `export`,
   `export_partial`, `import`, and an unsafe global `reset`; there is no public
   reachable-resource export API. `reset` is unsuitable while other owners are
   live. A standalone Rust probe linked the existing locked Symbolica rlib and
   constructed then dropped eight rational-polynomial coefficient Atoms with
   distinct `opaque(300-term-expression)` variables. Exporting the *empty*
   symbol set grew from **31 bytes to 32,264 bytes**, approximately 4,275 bytes
   per additional dropped coefficient after the first. This is measured retained
   serialization state, not an aggregate RSS measurement. The probe lives in
   ignored `output/serial-memory-review/partial_state.rs`.

   A second standalone probe generated, eager-compiled and indexed four
   distinct analytic endpoint integrals in both symbolic and numerical-dual
   modes. Empty-symbol state stayed **31 bytes** through all eight builds;
   archives ranged from about 3.9 to 4.6 KB. Those ordinary FastSecDec workloads
   did not exercise the retained polynomial tables. This distinguishes a
   demonstrated public native-input/host-history limitation from an unmeasured
   claim about current ggHH behavior. The second probe is
   `output/serial-memory-review/generation_state.rs`.

   Ordinary generation may similarly place unrelated historical resources in
   every independently readable sector record. Practical impact on ggHH and any
   necessary upstream resource-scoped API remain under investigation. Do not
   claim a strict active-sector memory bound merely because numerical evaluator
   objects have separate records.

3. **Serial worker reload forced expensive validation.** The initial resident
   worker used `KernelLoadOptions { validate: true }` regardless of the CLI
   selection. This silently changes the agreed default and may repeat heavy
   metadata/semantic proof checks on every reload. The coordinator now carries
   `validate_artifact` in the job and the worker uses it; independent source
   review confirms that mandatory identity/layout checks remain enabled.

4. **Failure recovery controls now cover the identified boundaries.**
   Independently inspected the new `durable_sector_without_receipt` test: it
   leaves a fully durable native sector file, removes its unacknowledged receipt,
   adds an interrupted `.writing` file, then reopens the journal. Data alone is
   rejected as completion; replay rewrites the sector, removes the temporary
   file, and the recovered indexed sector evaluates to the analytic value 2/3.
   This injects the precise filesystem state at the data/receipt boundary; it
   does not claim to interrupt a physical fsync instruction. The separate CLI
   test actually SIGKILLs the coordinator and resumes with a different worker
   count after the inherited lock is released.

   The new `/dev/full` control propagates real Linux ENOSPC through the atomic
   write closure after a partial temporary write, verifies unchanged published
   content, and verifies temporary-file cleanup. The existing staged-artifact
   failure control also verifies that the old immutable data stays readable.
   The coordinator's recorded licensed unit run passes these named controls
   (`cli-unit-refinement-tests.log`: 83 passed, 2 intentionally ignored); source
   review found no publication path that renames the manifest before syncing
   the complete data. These are local-filesystem failure controls, not a claim
   about network-filesystem durability or actual host disk exhaustion.

## Measurement protocol

Measure the same executable in fresh coordinator processes with small analytic
fixtures of increasing independent sector count and fixed worker count, then
vary worker count at fixed sector count. Capture coordinator RSS and each owned
child RSS separately and sum simultaneous observations. Keep source/preparation
peaks distinct from sector execution peaks, include load/JIT transients and
retain enough sampling frequency to observe short-lived workers. Report observed
peaks, never claim OS high-water marks from periodic sampling. Check all four
normal/serial generation/integration combinations, selected loading of a
nonzero sector ID, exact-only artifacts, and rich exact-chart records. A bounded
`gghh_double_box` run follows analytic checks; no triple-box run is required.


## Measured residency scaling

The independently run fixtures use `d = 4, 8` unit-cube parameters, singular
factor `(x0 + ... + x[d-1])^-1`, and asymmetric numerator
`1 + x0 + 2*x1 + ... + d*x[d-1]`. Saved catalogues were checked to contain
**4 and 8 distinct numerical sectors** respectively. This replaces earlier
one-dimensional trial inputs whose terms merged to one sector; those earlier
logs are not evidence of scaling with sector count.

Each measurement starts a fresh coordinator and recursively samples only its
owned process descendants through `/proc` at 20 ms intervals. Peaks are maxima
of simultaneous parent-plus-child RSS sums, including preparation, exact setup,
load/JIT and shutdown. Child counts include a process until its OS disappearance.
The native debug executable was hard-linked before the coordinator's subsequent
builds so replacement of `target/debug/fastsecdec` could not change the running
worker version. Settings were SymJIT O2, four 4096-point production replicas,
serial residence 0.01 seconds and one explicit production allocation per sector.
Both tolerances were zero; all four integration runs correctly stopped
**unconverged** at their work limit after complete coverage.

| Numerical sectors | Workers | Generation aggregate peak | Integration aggregate peak | Maximum observed children |
| --- | --- | --- | --- | --- |
| 4 | 1 | 69.3 MiB | 69.5 MiB | 1 |
| 4 | 4 | 191.5 MiB | 185.2 MiB | 4 |
| 8 | 1 | 69.8 MiB | 69.5 MiB | 1 |
| 8 | 4 | 192.7 MiB | 186.1 MiB | 4 |

The coordinator's observed peak remained 29.6–31.2 MiB. Doubling the sector
count did not double heavyweight residency at fixed worker count; increasing
workers increased aggregate memory. This agrees with the inspected process
ownership and selective-load boundaries. These small fixtures measure baseline
and active-owner scaling, not the peak size of an arbitrary hard sector. The
20 ms cadence may miss shorter peaks and is explicitly not an OS high-water
mark. Raw measurements are in ignored `output/serial-memory-review/scaling/`.

## ggHH native-state envelope inspection

Native staging envelopes were inspected without importing Atoms or restoring
any evaluator. Both symbolic and numerical-dual preparations contain 30 geometry
map records with exactly **31-byte state** each, establishing that the current
ggHH preparation does not carry historical polynomial/finite-field tables.
The source context is **1,893 bytes** alongside a roughly 4.24 MB source payload.
All 30 mapped dual charts have identical **1,373-byte state** alongside roughly
7.06 MB payloads. The four completed symbolic charts available from the stopped
bounded debug attempt have **1,446-byte state** alongside 3.13–3.22 MB payloads.
Those results distinguish small common symbol context from the much larger,
independently spooled chart expressions. The inventory is saved in ignored
`output/serial-memory-review/gghh-state-resources.json`.

The public Symbolica resource-selection caveat remains for callers with
rational-polynomial coefficient resources or substantial pre-existing native
state. It was not observed in these CLI ggHH or analytic records. This review
therefore accepts the measured CLI ownership/residency design, while retaining
that explicit upstream context limitation rather than promising that arbitrary
host-global state disappears when Rust owners are dropped.
