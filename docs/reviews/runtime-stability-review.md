# Runtime stability, weighted maxima and codec review

Independent source review by the dashboard/artifact agent, 2026-10-07.

Accepted the new runtime kernel routing and persistence after one public state
merge correction. The reviewed ownership boundaries are `kernel/stability.rs`,
`distance.rs`, `weighted.rs`, `precision_cache.rs`, `timing.rs`, the real/complex
kernel adapters, diagnostics aggregation, and format-7 artifact decoding.

Distance routing uses retained cancellation rows after coordinate mapping. It
combines endpoint logarithmic losses with precomputed normalized threshold
budgets, retaining per-axis source alternatives where original powers differ.
Inactive axes do not multiply a zero budget by an infinite endpoint logarithm.
Default thresholds choose native f64, native DoubleFloat at 106 bits, and native
arbitrary precision at 1000 decimal digits (3322 bits). The explicit validated
policy preserves its prior conditioning and adaptive precision checks. The
finite pure-imaginary/zero-component heuristic is bypassed only by the new
distance policy. Distance selection is a configurable numerical heuristic,
not a proof of relative accuracy or threshold safety.

All three levels evaluate the entire Laurent vector. Complex outputs retain
adjacent real/imaginary components; weighted escalation computes one `hypot`
magnitude per complex Laurent coefficient. It compares against prior local
final magnitudes and updates both retained maxima and the sequential reference
only after a finite final vector. Rejected lower-precision attempts cannot
change maxima. An explicit zero reference is skipped as required. Configurable
per-level escalation can advance through both higher levels; it cannot run
beyond the final arbitrary level. Weighting occurs once, at native precision
before conversion for the higher levels. No scalar component is independently
replaced or independently accumulated.

The review found that `WeightedEvaluationContext::merge_state` merged retained
maxima but reset the decision reference to the incoming, potentially smaller
state. The owner fixed it to copy the merged retained maxima. The separate
`set_reference_state` API deliberately retains its package-baseline replacement
semantics. The release-linked native probe confirmed that merging maximum 50
into maximum 100 retains reference 100 and leaves a subsequent value 60 at f64
under the 0.9 gate. Replay identity includes runtime binding and numerical policies;
caller-owned complete-work admission remains the checkpoint boundary.

Only a configured unstable cutoff fills outputs with zero and reports the
Unstable class without calling an evaluator. A nonfinite final arbitrary result
remains an error. Final class counts and attempted numerical-call timings are
separate: one sampled outcome belongs to one class, failures also count as
Unstable, and cutoff-zero and failure counts remain distinct. Missing historical
histograms remain unclassified. Driver error paths read cumulative evaluator
metrics before and after the call, preserving attempted work even when no
successful precision report is returned.

Timers enclose only the native numerical evaluator calls. They exclude input
conversion, cache construction, mapping, replay decisions and weighting.
Conditioning evaluator time has a separate bucket and is included in aggregate
evaluator work, but is excluded from the primary-f64 mean. Worker-local native
caches reuse the existing mapping requirements and evaluator owners; no new
arithmetic backend or numerical expression representation was introduced.
Validated arbitrary-precision calls can use variable precision, so human
report labels now distinguish `Arb (variable)` from distance-mode `Arb<1000>`.

Format 7 persists exact original endpoint profiles and includes them in semantic
identity. The loader has explicit version-5 and version-6 layouts, validates the
matching digest and semantic identity, and retains the original loaded bytes.
Absent historical profiles remain absent; they are not fabricated. Profile
validation checks dimensions, positive canonical native Rational powers,
nonzero orders, and equality of projected coverage with the existing
cancellation rows. Explicit Atom fields still use context-aware native binserde
with StateMap; the existing safe native evaluator serde adapter is unchanged.
Runtime stability settings do not rewrite immutable template bytes.

Supporting owner probes cover native analytic real/complex vectors, all three
classes, per-level whole-vector weighted replay, exactly-once weights, cutoff-only
zero, failed-attempt accounting, runtime binding, cold format-7 replay, historical
format-5/6 byte preservation, and legacy unknown counts. The root endpoint
probe additionally covers Taylor/IBP and physical/coefficient-series generation.
These are focused controls, not the deferred permanent test migration or a new
claim of convergence. The root standalone portable eager probe also passed all three native tiers;
its separate result record supplies execution evidence without claiming native
SymJIT cache compatibility or a browser deployment test.
