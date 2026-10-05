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
