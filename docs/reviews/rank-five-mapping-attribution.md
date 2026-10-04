# Rank-five mapping attribution and bounded reuse proposal

This diagnostic uses the actual production mapping path, with opt-in timers compiled only into unit-test builds. Normal library builds contain no timers, counters or profiling state. No support cache or production algebra change was made for this measurement.

The release probe uses the unchanged native rank-five box input and factored monomial stripping on Symbolica revision `98794d0d7337ba2b08e4c046dde584ad7fc1ce10` with the recorded essential patches, and SymJIT 2.26.4. It processes all 12 charts and complete requested Laurent coefficients. Our team's other runtime/build jobs were idle during the measured run. This is an internal generation diagnostic, not a matched Pathfinder performance result.

Raw evidence is `output/mapping-attribution-release.log` and `output/probes/rank-five-mapping-attribution-release.json`. The earlier sparse/factored report was preserved at its original path. The profile records per-chart, per-source-factor stage counts, durations, original Atom bytes and support sizes. No density expansion is introduced by instrumentation.

| Operation | Calls | Seconds |
| --- | ---: | ---: |
| Full generation | — | 11.360208 |
| Mapping, existing generation timer | — | 11.106182 |
| Native original-support extraction | 68 | 11.064826 |
| Native substitution | 144 | 0.008514 |
| Coordinate-face fast path | 144 | 0.018491 |
| Exponent transformation and minima | 68 | 0.000719 |
| Native common-factor collection | 68 | 0.008262 |
| Native polynomial recognition | 68 | 0.000312 |
| Residual/domain certification | 96 | 0.000621 |

No sparse residual fallback was used. Support extraction accounts for 99.63% of the mapping timer. All remaining generation stages total approximately 0.254 seconds. Native Gaussian parameterization (2.279 seconds) and the probe's separate input/equivalence validation (2.617 seconds) are outside that generation timer and are not hidden in the reported saving opportunity.

The four unchanged numerator factors each request their original support five times, once for each chart that needs nonzero-valuation analysis:

| Source factor | Atom bytes | Monomials | Repeated extractions | Total seconds |
| --- | ---: | ---: | ---: | ---: |
| 2 | 4812 | 258 | 5 | 4.718151 |
| 5 | 4440 | 258 | 5 | 4.475403 |
| 8 | 2680 | 245 | 5 | 0.888602 |
| 11 | 2942 | 245 | 5 | 0.982565 |

The other 48 calls collect a two-monomial singular factor and cost only about 0.000106 seconds combined. Repeating source support collection, rather than transforming support through each chart or collecting mapped factors, is the measured bottleneck.

## Proposed narrow implementation

Use a cache owned by one `generate` call, with the input's fixed ordered parameter vector and native `Atom` equality/ordering as the expression key. Cache only the exact `PolynomialSupport` returned by the existing Symbolica adapter. This reuses the existing CAS result; it introduces no polynomial representation, valuation algorithm, arithmetic or approximate equality. The support contains exact native exponents and is independent of chart, prefactor, role and regulator exponent once its source polynomial and parameter ordering are fixed.

Populate the cache only at existing support-request sites. In particular, retain the mapped coordinate-face zero-valuation fast path before asking for any regular factor's support. Do not eagerly enumerate high-degree compact numerator powers, and do not store mapped Atoms, residual proofs or failures. Singular supports already required for geometric decomposition may use the same generation owner. The cache is discarded on completion, cancellation or failure and cannot carry data across inputs or parameter orderings.

Keep exact exponent conversion bounds, signed infinity-chart handling, coefficient-aware adapter validation, residual certification, and direct factored output unchanged. The existing compact-degree-10000, hidden cancellation, orthant, literal-symbol and complete-vector scientific regressions remain relevant. A repeated-factor/different-exponent mathematical case can additionally guard against accidentally caching mapped density rather than source support. Measure the same bounded release probe after implementation, including retained sizes/hit counts; do not infer an achieved speedup from this attribution alone.

Production caching remains subject to coordinator and independent design review. Coefficient-specific periodization, alias-aware symbolic generation and catalog changes are separate tasks, not part of this proposal.
