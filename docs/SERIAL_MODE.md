# Sector residency and serial execution

Serial execution keeps each worker on one sector at a time. Workers still run
in parallel. The native library exposes synchronous jobs and scheduler state;
the CLI owns the processes and the integration loop.

```sh
fastsecdec generate run.toml --serial --workers 8
fastsecdec integrate integral.fsd --serial 60 --workers 8
fastsecdec run run.toml --serial 60 --generation-workers 8 --workers 8
```

`generate --serial` prepares file-backed native contexts and subtraction
formulas, then completes and saves each representative sector in a child
process. That child exits before the slot is reused. The coordinator retains
compact receipts and copies completed records without restoring evaluators.

`integrate --serial 60` gives each loaded sector a **minimum 60 seconds of
sampling time**. Loading, JIT, waiting and pauses do not count. Accuracy,
cancellation, numerical failures and explicit allocation limits may stop work
earlier. A complete QMC lattice or MC batch is indivisible. Each completed
replica updates the accepted estimate and status immediately, including while
a worker remains on the same sector.

The corresponding run-card settings are:

```toml
[generation]
serial = true

[integration]
serial_seconds = 60.0
double_points = false
workers = 8
# max_rounds = 4 # Optional per-sector production-allocation cap.
```

Both serial settings are disabled by default; `double_points` defaults to true.
Generation and integration settings are independent. `run --serial SECONDS`
enables both; using `integration.serial_seconds` alone leaves the generation
algorithm unchanged and isolates it in a child before serial sampling starts.

## Artifacts and inspection

Ordinary and serial generation publish the same indexed artifact format. The
stable `integral.fsd.json` manifest names an immutable, generation-specific
data file in the same directory. Keep that referenced file with the manifest;
do not assume its name is `integral.fsd.dat`. The manifest is replaced only after
the new data is complete and durable. An interrupted replacement leaves the
previous artifact usable. Older unreferenced data files may be removed when no
reader needs them.

Both integration modes can use either generation mode's new artifact. The
supported older monolithic format remains readable by ordinary integration.
Serial integration asks for regeneration when given a monolithic artifact.

`inspect integral.fsd` is metadata-only. `inspect integral.fsd --sector 3
--deep` restores only sector 3, retaining its local coefficient layout and
mapping to the complete Laurent vector. `--expressions` exposes its retained
change of variables and subtraction metadata. Expensive validation remains
opt-in with `--validate-artifact`, including resident-sector reloads.

Newly compiled records include the native primary JIT payload as well as the
exact program. A compatible payload restores through Symbolica without another
exact-to-JIT translation; older records or incompatible primary caches prepare
the backend from their admitted exact program. Native application restoration
still compiles executable code. Symbolic generation and Horner/CPE are not
repeated. Selected-record loading and caller-owned residency are unchanged;
see [the native cache contract](DEVELOPMENT.md).

## Coverage, refinement and uncertainty

The first sweep assigns distinct sectors. A valid first visit needs at least
two complete production replicas and the residence minimum, unless an explicit
stop condition ends it. Waiting workers can continue their own resident work.
After complete first coverage, the scheduler prioritizes the sector contributing
the largest uncertainty to the requested accuracy target. It keeps a winning
resident loaded, including on equal priority. Multiple workers may serve the
same sector after coverage.

Serial QMC uses independent randomizations across sectors and adds their full
covariance matrices. Ordinary democratic QMC keeps its shared-shift covariance.
Serial `mc` and `adaptive_mc` use per-sector native Havana grids. Adaptive pilots
are excluded from estimates, and production uses frozen grids. Serial
`discrete_mc` is rejected; choose per-sector Havana or ordinary discrete MC.

With `double_points=true`, successive allocations grow lattice or batch sizes.
QMC extends independent shifts when its lattice size reaches the catalogue
limit. With `double_points=false`, sizes stay fixed and the cumulative replica
target doubles; only additional independent replicas are sampled. Fixed-size
continuation preserves frozen production grids and does not repeat pilots.

Different lattice sizes and changed proposals have separate statistical epochs.
The preceding completed estimate remains identifiable while its replacement is
incomplete. Missing estimates, pilots and sampled zeros never stand in for exact
contributions. Full real/imaginary and Laurent covariance is preserved.

In serial mode, omitted `max_rounds` permits unlimited refinement. An explicit
cap counts each sector's production allocations, including the initial one;
residence visits do not consume allocations. Ordinary mode keeps its historical
one-allocation default. Exhaustion without reaching tolerance is a work-limit
stop, not convergence.

## Seeds and recovery

Only the coordinator reserves sampling work. A reservation binds the integral,
sector, phase, epoch, replica and native random stream. Worker index, PID and
restart never determine seeds. Checked stream/draw counters reject exhaustion.
Independent same-sector workers receive distinct complete lattices or MC
batches. Retrying interrupted work retains its identity only after the old
worker is confirmed dead; stale or duplicate completions are rejected.

Integration keeps the periodic checkpoint cadence. Resume may change worker
count, minimum residence and evaluator batch size. It preserves durably accepted
statistics, replay state and stream frontiers. Uncommitted work lost in a crash
may be replayed, but cannot run simultaneously or enter an estimate twice.
Serial and ordinary checkpoints describe different statistical models and are
not interchangeable.

Interrupted serial generation retains a sibling `.generation` staging
directory. Resume with `fastsecdec generate run.toml --resume --output
integral.fsd`; it validates receipts, input and build identity before reuse.
Partial staging is never a complete artifact. Stable OS lock files are shared
with the child processes, preventing a new coordinator from reissuing work while
an old child remains alive. Do not remove lock files to bypass that check.

## Observations and validation

Status snapshots expose coverage, resident PID/sector, sampling residence,
loading time, epoch, replica progress, error priority and checkpoint age. The
dashboard samples aggregate coordinator plus owned-child RSS, rather than only
the coordinator. The memory budget includes compact coordinator state,
necessary native source/context overhead, and active-sector decoding/JIT/scratch
peaks. Process exit releases native allocator state; dropping Rust values alone
does not establish an OS memory bound.

The required acceptance matrix is in [SERIAL_MODE_PLAN.md](../SERIAL_MODE_PLAN.md).
Independent findings and limitations are recorded in the
[sampling review](reviews/serial-seed-statistics-independent.md) and
[artifact/memory review](reviews/serial-artifact-memory-independent.md).
