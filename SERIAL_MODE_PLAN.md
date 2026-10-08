# Bounded-memory sector generation and integration

## 1. Interface and agreed behavior

Add sector-at-a-time execution while retaining caller-owned scheduling and existing numerical methods.

```bash
fastsecdec generate run.toml --serial --workers 8
fastsecdec integrate integral.fsd --serial 60 --workers 8
fastsecdec run run.toml --serial 60 --generation-workers 8 --workers 8
```

- `generate --serial` completes, persists and releases each sector before assigning another to that worker.
- `integrate --serial <seconds>` sets a **minimum sampling residence time** per loaded sector, excluding loading/JIT time and pauses.
- `run --serial <seconds>` enables both behaviors.
- Workers remain parallel: “serial” describes sector residency, not a single-worker restriction.
- **All generation paths write the same universal artifact format.** Either integration mode can consume any newly generated artifact, regardless of generation scheduling.
- Add runtime settings `generation.serial`, `integration.serial_seconds`, and `integration.double_points`. The first two default to disabled; `double_points` defaults to `true` and applies to ordinary and serial integration.
- In serial integration, omitted `max_rounds` means unlimited refinement. An explicitly configured limit applies per sector and counts production/refinement allocations, including the initial allocation—not residence visits. Ordinary integration retains its current default limit.

The final scheduling policy replaces the earlier hard 1% gate and fixed `sqrt(N)` rule. Global convergence, cancellation, failures and explicit work limits can end work before the minimum residence time.

**Seed and work partitioning are scientific correctness requirements and mandatory acceptance gates. Multiple workers serving the same sector must never run the same assigned random stream or repeat the same sampling work.**

## 2. Universal artifacts and bounded-memory generation

Introduce a versioned, indexed artifact format with independently readable sector records.

- Preserve CLI artifact basenames. The metadata manifest references an immutable, generation-specific data file and records sector offsets, lengths and digests. Publish the manifest atomically after completing the data file; interrupted replacement must leave the previous artifact usable.
- Each sector record contains its optimized native evaluator program and associated rich inspection metadata. Keep global input identity, runtime parameters, constraints, exact offsets and compact sector descriptors separate.
- Reuse Symbolica’s existing native serialization and evaluator codecs. Metadata-only inspection remains lightweight; selected-sector inspection loads only the requested records.
- Normal and serial generation use the same writer and canonical sector ordering. Scheduling choices do not enter mathematical identity.
- Existing monolithic artifacts remain usable through their supported ordinary reader. Serial integration rejects them with an explicit regeneration instruction; no conversion tool is included.

Refactor generation into shared preparation followed by a consuming sector pipeline:

1. Prepare the immutable source context, geometry, exact symmetry representatives and unique subtraction-formula requirements.
2. Spool heavyweight preparation products and reusable formulas to disk. Keep only compact indices and construction bookkeeping in the coordinator. Preserve native exact symmetry verification; hashes only locate candidates.
3. A worker imports the required context and completes one representative sector through subtraction, Laurent construction, evaluator optimization/compilation and durable serialization.
4. Return a small completion receipt, release the worker, and admit another sector. Never queue completed evaluator objects or accumulate all sector expressions in the coordinator.
5. Finalize the common coefficient layout, exact contributions and manifest without restoring completed sectors.

Sector records carry their local Laurent/component layout. The final catalogue supplies mappings into the complete global vector, allowing earlier sectors to remain compiled when later sectors introduce additional orders or complex components.

Retain formula reuse through immutable disk-backed records. Heavy source/jet/template caches must have explicit lifetimes; they cannot remain as an unbounded cache in another process.

## 3. Process residency and integration scheduling

The CLI owns a pool of at most `workers` child processes. The native library exposes synchronous jobs, selective loading and scheduler state without owning threads or processes.

- Generation workers exit after persisting their sector.
- Integration workers remain alive while their current sector is useful. On eviction, checkpoint accepted sampling state, terminate the process, confirm its exit, and only then start its replacement.
- Load one sector’s saved optimized program and bind its parameters. Reloading may repeat SymJIT backend translation, but must never repeat symbolic generation or Horner/CPE optimization.
- Heavy payloads travel through artifact/staging files. IPC carries bounded commands, progress and completion receipts.

The honest memory bound is:

**compact coordinator state + necessary source/context overhead + at most `workers` active-sector peaks**, including decoding, JIT and evaluation scratch space. Measure aggregate parent-and-child RSS; do not equate Rust object destruction with memory returned to the OS.

### First coverage and subsequent scheduling

- Initially assign distinct unvisited sectors. A first visit completes after the minimum residence time and sufficient complete replicas to provide an uncertainty estimate—at least two independent production replicas.
- Exact contributions bypass sampling. Missing estimates, pilot-only observations and sampled zeros never count as exact zero contributions.
- Do not enter error-priority scheduling until every required stochastic sector has completed its first visit. Workers reaching the barrier can continue their resident work; do not allocate additional workers to another sector before coverage is complete.
- Thereafter select the largest error contributor to the requested accuracy target. For one complex Laurent coefficient, use the sum of its real and imaginary variances. For all-component targeting, normalize by the existing tolerance scales, with explicit zero-threshold handling.
- Reconsider assignment at complete lattice/batch boundaries once the residence minimum has elapsed. Prefer the resident sector on equal priority, then stable sector ID.
- If the resident sector still wins, continue without unloading. Its residence clock does not restart. Multiple workers may then serve the same sector.

**Every accepted complete lattice or MC batch updates the coordinator immediately**: sector statistics, global running estimates, priorities and streamed status. Updates are independent of residence expiry. Partial allocations remain visibly provisional; final convergence requires complete coverage and valid production evidence.

Serial QMC uses independent sector randomizations and adds sector covariance matrices. Serial MC uses per-sector Havana sampling, preserving native pilot/freeze behavior for adaptive variants. `discrete_mc --serial` is rejected with guidance to use per-sector Havana; ordinary discrete MC remains available.

## 4. Refinement, seed safety and recovery

Apply `double_points` consistently across integration methods:

- **Enabled:** retain point-count growth for QMC lattices and MC batches. Refine sectors independently in serial mode. At the QMC catalogue limit, extend independent shifts instead.
- **Disabled:** keep lattice/batch sizes fixed and append genuinely new independent replicas. Each refinement doubles the cumulative replica target, computing only the additional replicas.
- Fixed-size continuation retains frozen production grids and prior production statistics. It does not rerun pilots or reuse old random streams.
- Different lattice sizes or changed production designs remain separate statistical epochs. Never pool coarse and fine evaluations as independent replicas. Retain the preceding complete estimate while its replacement is incomplete, and distinguish it from current live progress.
- Ordinary democratic QMC retains its intentional shared-shift covariance across different sectors when appending replicas. Ordinary discrete MC retains its frozen global proposal when appending batches.

### Seed safety: enforced, not assumed

The coordinator exclusively reserves complete shifted lattices or MC batches. Each reservation binds the integral, sector, pilot/production phase, sampling epoch, replica ID and random-stream identity.

- **Never seed workers independently from the same master seed.** Worker number, process ID, wall-clock time and process restart must not determine or reset sampling streams.
- Use the existing native RNG stream-partitioning facilities, after verifying their jump spacing, period and maximum permitted draws per reservation. Enforce checked counters and reject exhaustion or overflow rather than allowing stream reuse.
- Concurrent workers serving one sector receive distinct QMC randomizations or disjoint MC streams. Within a QMC replica, point-index ranges are uniquely owned; two workers cannot own the same sampling range.
- Eviction, reloading, continued residence and checkpoint restoration preserve the coordinator’s stream frontier. Appending replicas always allocates new identities and streams.
- Retrying interrupted work retains its original identity. Confirm that its previous worker is dead before reissuing it; fence late returns with run and lease identities.
- Reject duplicate, unissued and stale completions. Commit statistics, completion identity, replay state and diagnostics together, so a replica cannot enter the estimate twice.
- Drain old-epoch reservations before changing a sector’s lattice or grid. Do not count nested coarse/fine sampling as independent evidence.
- Audit **actual generated coordinate sequences and ranges**, not merely different task IDs or seed integers. Tests must expose repeated point sequences caused by reseeding, overlapping streams or incorrect restoration.

This guarantees unique assigned sampling work and prevents duplicated worker streams. Individual chance coordinate coincidences between independent finite-precision random draws are distinct from running workers on the same sample sequence.

Compact completed replicas into numerically stable vector sufficient statistics using existing Numerica primitives. Confirm any missing accumulator operation through the required API/source/probe audit before adding a narrow helper. Bound outstanding work and out-of-order returns so bookkeeping does not grow with run duration.

Reuse the existing periodic checkpoint cadence, adding residency and reservation state. Resume permits changed worker counts and residence times. It preserves durably accepted work and random-stream frontiers; crash-lost, uncommitted work may be replayed, but never simultaneously or accepted twice.

Generation gains resumable staging receipts and `generate --resume`. Partial generation is never accepted as a complete artifact. Integration checkpoints require a matching statistical mode; universal artifact compatibility does not make shared-shift and independent-sector checkpoints interchangeable.

Explicit budget exhaustion reports an unconverged result. It never becomes a successful accuracy stop.

## 5. Implementation goal, delivery and acceptance

**Upon starting implementation, write this complete approved plan into `SERIAL_MODE_PLAN.md` at the repository root, reference it from `FIRST_PHASE_PLAN.md`, and assign its implementation and acceptance gates to the assistant as an active goal before changing implementation code.**

The goal statement is:

> Implement `SERIAL_MODE_PLAN.md` completely: deliver universal sector-addressable artifacts, bounded-memory serial generation and integration, configurable refinement, rigorously partitioned sampling streams, correct statistics and recovery, and validated CLI/native-library interfaces. Complete the acceptance gates, commit and push validated milestones, and mark the goal complete only when all required work is finished.

Delegate separate implementation slices for artifact storage, streaming generation, process execution, integration/refinement, and status presentation. Assign independent reviews of scientific statistics, **seed/stream partitioning**, memory ownership and HEPKit reuse before accepting milestones. Keep substantive functionality in Rust/FastSecDec and commit validated milestones on `main`.

Extend status structures and CLI views with coverage, resident sector/process, residence age, load/JIT time, refinement epoch, pending uncertainty, priority and checkpoint state. Presentation consumes snapshots rather than borrowing a running native session.

Acceptance requires:

- All four normal/serial generation–integration combinations using the same artifact format.
- Scientific parity across symbolic and numerical-dual generation, Taylor/IBP, complex Laurent vectors, symmetry multiplicities, runtime parameters and exact offsets.
- First-coverage priority, minimum-residence enforcement, live updates during residence, retained winning sectors and concurrent same-sector workers.
- Both growth settings across serial and ordinary QMC/MC, including ordinary discrete MC.
- **Dedicated duplicate-work regressions with multiple workers on the same sector**, inspecting generated sampling sequences under reordered returns, process replacement, retries, both refinement policies, and checkpoint/resume with changed worker counts.
- **Independent verification of native RNG partitioning and reservation limits**, including counter exhaustion, stale leases and attempts to submit the same replica twice.
- Large-cancellation and near-zero controls that preserve full covariance and never fabricate missing contributions.
- Fault-injected interrupted writes, corrupt records, disk exhaustion and generation recovery.
- Instrumented ownership checks and measured aggregate RSS across increasing sector counts: heavyweight residency must scale with worker count, not the total sector count.
- A bounded native ggHH demonstration after smaller analytic tests; no expensive three-loop run as the initial validation gate.
