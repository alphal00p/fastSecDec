# Independent initial Numerica QMC review

Reviewed commit `e26d3dd3ee0683c5acd9706eb95fb6b66f24147b` on 2026-10-04. Scope: the separate `numerical_integration::qmc` lane, randomization and worker packages, shifted-lattice statistics, merges, checkpoint validation, and periodization. This review does not establish performance parity with the reference implementation.

## Findings requiring correction

1. **P2: large shared offsets bias shift covariance.** In `src/numerical_integration/qmc/statistics/estimate.rs:63–71`, deviations are taken from the rounded absolute mean. Exactly representable observations `1e16` and `1e16 + 2` have covariance of their mean equal to 1, but the current API returns 2 and standard error `sqrt(2)`. Compensated summation of absolute observations does not fix the final mean rounding. Compute centered offsets about a nearby anchor, then use the centered mean for covariance. Preserve the full output covariance, including off-diagonal entries.

2. **P2: shift addition rounds before periodic reduction.** In `src/numerical_integration/qmc/work.rs:219–221`, allowed rule `n=2^53, z=[1]` with shift `[0.75]` maps both lattice indices `2^51` and `2^51+1` to zero. Their shifted coordinates should be `0` and `2^-53`, respectively. Modular integer multiplication is correct, but a floating addition above one destroys the last bit before wrapping. Implement a reduction which retains the supported binary lattice resolution, and test nonzero shifts at the largest accepted modulus. The published Kuo catalog uses smaller moduli; this finding concerns the public custom-rule range.

Both findings were reported to the author and coordinator before any dependency edits. Reproductions below were compiled directly against the workspace's current Numerica rlib and executed successfully; no Numerica source was modified by the reviewer.

```rust
use numerica::numerical_integration::qmc::{QmcEstimate, QmcPlan, Rank1Rule};
let estimate = QmcEstimate::from_shift_means(
    &[vec![1e16], vec![1e16 + 2.0]],
).unwrap();
assert_eq!(estimate.covariance_of_mean, vec![1.0]); // actual: [2.0]

let n = 1u64 << 53;
let plan = QmcPlan::with_shifts(
    Rank1Rule::new(n, vec![1]).unwrap(), vec![vec![0.75]],
).unwrap();
let mut point = [0.0];
plan.point((1u64 << 51) + 1, &mut point).unwrap();
assert_eq!(point, [1.0 / n as f64]); // actual: [0.0]
```

## Reviewed invariants

- Random shifts have stable prefixes under changes in dimension and shift count. Worker boundaries do not influence point coordinates. Workers can generate point buffers locally from immutable plans and interval descriptors.
- Custom rules validate coprimality and supported modulus. Products use wrapping arithmetic only for power-of-two moduli, where the subsequent mask is exact, and use `u128` modular products otherwise.
- Merge validation checks plan identity, output dimension, complete packages, predecessor/successor interval overlap, and checked interval bounds before insertion. Nonoverlap and full per-shift point counts establish completeness; incomplete shifts are excluded from estimates.
- Reductions use a canonical package ordering and compensated sums. Fixed packages produce results independent of arrival order. Different package sizes may change floating sums, as documented.
- Statistical estimates use complete independent random-shift means, with covariance divided by `m(m-1)`. The API documents summing correlated sector vectors by shift identity before statistical reduction. Raw lattice points are not treated as independent observations.
- Checkpoints reconstruct rules, plans, partials, and accumulators through validation. Bit-preserving shift/sum storage avoids changing a resumed plan. Plan fingerprints detect accidental mixing; they are explicitly not authentication.
- Korobov-3 uses the correct derivative and reflection for the upper half interval. The caller controls scheduling and parallelism; no Monte Carlo API was replaced.

The author's 21 existing QMC tests were read, including covariance, partial shifts, package gaps/overlap, checkpoint corruption, correlated-sector cancellation, and an ensemble error calibration. The author reported those passing; this review independently executed the two additional counterexamples above. Corrections and regression tests should be recorded below before this review is considered closed.

## Resolution

Both findings are resolved by Numerica commit `e9b7481d66b8f9d0c5e58ccabc4d6644e7fd479a`. The independent reviewer inspected the complete patch and executed the same standalone API reproductions against the updated Numerica library: covariance is now `1.0`, standard error `1.0`, and the second shifted point is exactly `1.1102230246251565e-16 = 2^-53`. New author regressions cover off-diagonal covariance, large constant means, multiple maximum-modulus shifts, sub-lattice residues, and an addition just below one that rounds upward. These address the causes, not just the two original examples. No unresolved findings remain from this initial review.
