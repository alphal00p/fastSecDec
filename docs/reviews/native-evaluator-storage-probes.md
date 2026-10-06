# Native evaluator storage and arithmetic probes

These bounded diagnostics use the accepted scalar-double-box Taylor artifact
and the `e25ca59` native scientific implementation. They do not change production
defaults, artifact formats or the precision policy. They reuse Symbolica's public
native evaluator APIs, without reconstructing or expanding the original atoms.

## Arithmetic optimization: rejected

Merging each evaluator with an empty evaluator of the same ordered parameters,
using Symbolica's public `merge(..., Some(1))`, runs its existing common-expression,
common-pair and stack optimizations. All 102 evaluators passed native layout,
constant/function, validation and cold-round-trip controls. Aggregate arithmetic
operations fell from 87,174 to 66,144; encoded native programs fell from 815,231
to 529,491 bytes. These counts alone do not establish a useful optimization.

The subsequent physical gate failed at the 38th complete vector, sector zero,
point 37. The finite weighted value's relative error against the original native
weighted MPFR oracle increased from 5.57e-13 to 1.024e-12, exceeding the unchanged
1e-12 relative gate. Both evaluations used ordinary binary64 without rescue at
that point. The oracle agreed at 128 and 256 bits. The remaining 7,714 planned
vectors were not visited. No timing, production adoption, tolerance change or
retry followed. This rejects this candidate under the existing policy; it does
not establish a general defect in Symbolica's arithmetic optimizer.

Evidence: `output/diagnostics/double-box-native-ir-optimization-{1,2}/`, including
the unchanged native source bindings and independent source/result reviews.

## Storage optimization: invariant on the retained sample

The separate candidate calls only public `optimize_stack()`. It preserves the
arithmetic counts and exported instruction-kind sequence, constants, functions,
parameter/output layouts and native round trips for all 102 evaluators.

| Aggregate over all 102 evaluators | Original | Storage candidate |
| --- | ---: | ---: |
| Arithmetic operations | 87,174 | 87,174 |
| Exported instructions | 40,373 | 40,373 |
| Temporary slots | 40,175 | 9,354 |
| Encoded native program bytes | 815,231 | 491,211 |

These are totals, not per-sector sizes or SymJIT machine-code measurements.
The candidate reduces temporary slots by 76.7% and encoded native program bytes
by 39.7%. Every complete vector and every precision/check/replay decision is
bitwise equal across 7,752 retained weighted points: 6,528 points from the existing
HKKN/Korobov2 prefixes and 1,224 axis-boundary controls. Both versions perform
2,601 rescues, 117 weighted checks and 75 additional replays, reaching 1,280 bits.
The sole runtime takes 55.197 seconds including supervision. All 424 frozen
inputs remain unchanged and all owned processes are reaped.

Independent original-IR MPFR evaluation also exposes an existing ordinary-lane
limitation: both versions have the same 60 vectors with at least one component
outside the configured 1e-12-relative plus 1e-300-absolute comparison. These are
34 prefix and 26 synthetic-boundary vectors, with 61 components affected. All
use unchecked binary64 in both versions. There are no new candidate violations.

The largest absolute discrepancy over the full sample is 4.1041e-11 for a finite
value about -219.26; that comparison passes its relative tolerance. Among the
violating components, the largest absolute discrepancy is 2.1239e-11 for a finite
value about -2.74677, with relative error 7.7325e-12. The largest finite relative
discrepancy is 1.8337e-11. All 27 finite-order violations occur in the native QMC
prefixes. Near-zero pole components additionally produce large relative ratios.
The failures are retained: neither the existing ordinary precision heuristic
nor this storage-invariance check is a universal point-accuracy certificate.

Evidence: `output/diagnostics/double-box-native-stack-1/`, including the complete
vectors, native oracle comparisons, `baseline-accuracy-summary.json`, assessment
and independent review. The independent review SHA-256 is
`bb9d62ef0987091f50da0318d80ddfb50158aa6c215d0df186b4a37ded0cbe37`.
One separate bounded full-workload timing comparison is authorized before any
production decision. Storage reduction is not yet a speed or parity claim.
