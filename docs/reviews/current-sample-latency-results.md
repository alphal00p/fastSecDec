# Current individual-sample latency

The current triangle diagnostics complete on one physical core with the ordinary
precision policies enabled. These are individual API-call observations, separate
from the [seven paired eight-worker measurements](minimal-paired-acceptance.md).
No additional triangle optimization is needed for the measured comparison.

| Triangle metric | FastSecDec | Pathfinder |
| --- | ---: | ---: |
| Timed rows | 262,144 | 262,144 |
| Sample-weighted mean, µs | 5.927280 | 58.480256 |
| Observed individual maximum, µs | 141.361 | 1,013.965 |

Each program retains two sector IDs with 131,072 timed rows each. FastSecDec's
sector means are 4.160120 and 7.694439 µs, with maxima 76.660 and 141.361 µs.
Pathfinder's means are 53.701925 and 63.258586 µs, with maxima 1,013.965 and
861.633 µs. Sector IDs are local to each program; this does not assert a
pointwise correspondence between their sector representations.

FastSecDec times the weighted complete-vector kernel callback, including
conditioning, MPFR rescue and weighted replay. Pathfinder times its actual
single-row `explicit_qmc_component_batch` dispatch through native selected-column
weighting, including Python/NumPy allocation and two narrow hook assignments.
Both exclude point generation, transformation, shift reduction, loading and
recording; Pathfinder's global prefactor is also outside its bracket. Its
production evaluator accepts vectorized batches, so the single-row diagnostic
is not production throughput. Finite observed maxima include interruptions and
are not worst-case bounds.

Both use the accepted triangle inputs, N8192, R16, seed20261302, Kuo33002 and
Korobov3. The eight-worker ordinary row supplies scientific/input admission;
the diagnostic changes only worker count to one on CPU0. Runs are serialized.
The authorized license is inherited without recording its value or private
configuration path.

FastSecDec's instrumentation-off/on runs agree exactly in the complete vector,
covariance, shift estimates, accepted replay state, precision diagnostics and
per-package output digests. All 262,144 calls succeed, with 80,039 rescues,
maximum precision 320 bits and two additional replays. Its separate rejected
seven-call prefix leaves accepted state unchanged and the native checkpoint
retry succeeds. The timed maxima both used 256-bit rescue. Pathfinder's separate
one-worker off/on processes agree exactly in physical vectors, errors, sector
work and precision; every replayed row equals its ordinary batch output before
the native shift reduction is checked. Its recorder is unchanged.

No clock overhead is subtracted. The 4,096 native empty brackets have mean
29.362 ns and median 30 ns; Pathfinder's 1,000 have mean 73.802 ns. Native caller
loop times are 1.621321 seconds off and 1.640419 seconds on. Reference whole
process times are 4.422/33.594 seconds, including replay and validation work;
these different boundaries cannot be compared as instrumentation overhead.

The native caller reuses accepted binary `b1a92557…` and its unchanged native
mathematical libraries. A separate compatibility record binds CLI inspection
to release `b772570e…`; there is no native rebuild, estimator change or precision
change. All processes exit zero without timeout and their identity checks pass.

Evidence is under
`output/benchmarks/native-current-sample-latency-plan3/triangle/` and
`output/benchmarks/pathfinder-current-sample-latency-plan3/triangle/`.
The combined summary SHA-256 is
`ae8f388a6137a35022d634f62ece05badb51a9ab298ace4b24d8ec314d19c545`.
The independent paired/reference review is
`minimal-paired-current-3/triangle/independent-review.json`
(`c68e998e…`). Pathfinder independently recomputed the native diagnostic and
accepted its exact controls and statistics in the sibling
`triangle/independent-native-review.json`. The native real ABI remains a three-component real vector with a
3×3 covariance. Its missing imaginary coordinates are not fabricated: the
Euclidean input, existing analytic Gamma identity and HEPKit C0 tests provide
the separate reality/reference evidence. The reference complex union passes.

## Box

The same diagnostic completes for the accepted box input and first paired row,
with N8192/R16/seed20261302 and one worker on CPU0. Each program records 393,216
rows: 131,072 for each of three local sector IDs. Both off/on controls pass,
including the native full covariance/replay/output-digest comparison and the
reference physical-vector/error/precision comparison. Native rejected-prefix
retry also passes. No sampler, precision threshold or evaluator is changed.

| Box metric | FastSecDec | Pathfinder |
| --- | ---: | ---: |
| Sample-weighted mean, µs | 6.766568 | 69.559459 |
| Observed individual maximum, µs | 4,037.015 | 1,583.796 |
| Local sector 0 mean / maximum, µs | 7.795175 / 4,037.015 | 79.896452 / 1,583.796 |
| Local sector 1 mean / maximum, µs | 5.252953 / 152.461 | 64.442037 / 653.963 |
| Local sector 2 mean / maximum, µs | 7.251576 / 3,997.945 | 64.339888 / 655.142 |

The native mean is lower, while its observed maximum is higher. The maximum
records a 320-bit rescue in sector 0, shift 7, lattice index 6316, with a
transformed coordinate about 1.50×10⁻²⁰ and weight about 3.56×10⁻²⁰. Sector 2's
3.998 ms maximum also uses 320 bits near a coordinate of 3.48×10⁻²³. Lazy
precision-cache construction is inside the callback, but these recordings do
not separate its cost, repeated arithmetic and scheduling interruptions.
Neither maximum is discarded or attributed to a scheduling gap. The paired
generation/integration medians and this single-run maximum remain distinct
metrics; no worst-case or tail-parity claim follows.

Native diagnostics retain 94,949 conditioning checks, 79,731 rescues, maximum
320 bits, three additional replays and zero failures. Its off/on caller loops
take 2.732632/2.805162 seconds. Pathfinder's whole off/on processes take
7.056295/58.593569 seconds; the instrumented process includes single-row replay
and validation. Its 1,000 empty clock brackets average 93.697 ns, with no
subtraction. All processes are reaped successfully and identity checks pass.

The successful reference command uses its existing target-file option to load
the exact stored pySecDec target also transported in `examples/targets/box.json`.
The failed attempt to invoke the unavailable OneLOop provider remains intact.
The diagnostic copies the accepted command and changes only worker count and
result destination, retaining that target choice.

Evidence uses the same roots as triangle, under `box/`. The combined summary
SHA-256 is `495835bb7e4e47e5743c43e696e493b31665ae3afc4cdbf7995c606b0603fb26`.
The independent seven-pair review is
`minimal-paired-current-3/box/independent-review.json` (`373f12ec…`), and the
reference latency review is `pathfinder-current-sample-latency-plan3/box/independent-review.json`
(`4263ba43…`). Pathfinder independently accepted the native controls and
recomputed all native statistics in the sibling `independent-native-review.json`.

### Corrected first-use measurement

The historical box table above remains intact. Source inspection found that the
Pathfinder recorder evaluated each ordinary batch before timing its individual
rows. That warmed lazy evaluation state outside the recorded interval. The
corrected recorder times every row first, then performs the unchanged ordinary
batch equivalence check. The native recorder likewise completes its timed pass
before its separate equivalence pass. No evaluator, precision policy, sampler,
point or production cache changed.

One fresh complete-stream observation, serialized on CPU0 after the owned Wasm
build and browser processes had finished, gives:

| Box metric, corrected timed-first ordering | FastSecDec | Pathfinder |
| --- | ---: | ---: |
| Timed rows | 393,216 | 393,216 |
| Sample-weighted mean, µs | 9.878479 | 70.390258 |
| Observed individual maximum, µs | 5,863.705 | 43,051.430 |
| Local sector 0 mean / maximum, µs | 11.373119 / 5,863.705 | 65.974259 / 43,051.430 |
| Local sector 1 mean / maximum, µs | 7.679639 / 169.800 | 78.482209 / 36,677.323 |
| Local sector 2 mean / maximum, µs | 10.582680 / 5,726.054 | 66.714305 / 35,214.030 |

FastSecDec has lower mean and maximum in this corrected observation. This is a
single finite sample of elapsed times, not a tail-distribution comparison or a
worst-case guarantee. In particular, Pathfinder's 43.051 ms outlier is retained;
this recorder has no per-call CPU measurement that would distinguish arithmetic,
lazy preparation, allocation, garbage collection or scheduling. No specific
cause is inferred. Local sector IDs still do not imply cross-program sector
correspondence.

The earlier three-point cold/warm diagnostic separately confirmed a real native
first-use cost: one retained difficult point took 4.403 ms wall / 4.262 ms thread
CPU on its first call and about 36 µs on warm calls. Pathfinder's corresponding
first call took 5.042 ms wall / 3.905 ms CPU and about 738 µs warm. That experiment
included fresh application owners, not guaranteed fresh process-global state,
and ran alongside compilation; it explains why charging first use matters but
is not the full-stream parity evidence. Its other observations and full-vector
normalization checks remain in `box-cold-warm-attribution-1/`.

The corrected full-stream run retains all 384 task digests, complete vectors,
covariance, shifts, precision decisions and replay state. The entire native
scientific record equals the historical record exactly, including 79,731
rescues and three additional replays. Every nontiming field in all 384
Pathfinder batches also equals its historical record, and all row-to-batch
vector/precision checks pass. The native rejected-prefix retry still passes.
Both processes exit successfully and are reaped; their measured whole-process
walls are 10.145 and 63.430 seconds, which include different validation work and
are not throughput ratios.

The native helper links the accepted archived optimized a3/f6 mathematical
libraries, with the already documented equivalent Option-spelling difference
from published source. The old artifact's original reader supplies provenance
admission; numerical work uses the bound current libraries. The one-active-sector
context policy remains unchanged. Loading and context construction remain
outside sample brackets in both recorders; preparation triggered inside a first
evaluator call remains inside. Complete-vector and global-prefactor boundary
qualifications above continue to apply.

The accepted retained-data assessment and all immutable source/process bindings
are in `output/diagnostics/box-fullstream-first-use-1/assessment.json`. The earlier
historical output has not been overwritten. No further box timing or cache change
is needed for this corrected observed metric.
Independent review in the same directory (`independent-review.json`) recomputes
all sector/total statistics, checks every canonical task/batch and accepts the
preserved full-vector equivalence and measurement boundaries.
