# Original hard four-loop Pathfinder generation

The single bounded Pathfinder attempt completed the original full-positive-orthant
input and passed its strict prepared-bundle reader. This establishes generation
capability; integration, one-per-mille convergence and matched sample latency were
not measured.

The input remains the original nine-variable `U * F^(eps-3)` density with unit
coordinate measure and global prefactor, `geometric_infinity_no_primary`, symmetry
disabled, and the existing projector/IBP endpoint treatment. Native provider
parsing reproduced the independently audited UF input exactly. Explicit complex
SymJIT O2 evaluators retain every provider order `[-8,...,0]`; no lower coefficients
were discarded or declared zero.

| Completed stage | Observed seconds |
| --- | ---: |
| Input admission | 0.329 |
| Ordinary generation process | 524.117 |
| Strict bundle hydration | 2.747 |
| Complete guarded attempt, including supervision | 528.820 |

The completed bundle contains 3,728 nine-dimensional sectors, 3,728 explicit
sector formulas and 11,222 evaluator files. Its native timing record attributes
1.972 seconds to decomposition, 5.973 to sector conversion, 508.823 to explicit
sector construction and 5.499 to serialization. The broader log interval around
explicit construction is 508.873 seconds; these two instrumentation boundaries
are retained separately.

The sole attempt used CPU0 with a 600-second budget and five-second interruption
grace. Sampled aggregate owned-process RSS peaked at 1,184,411,648 bytes
(1.103 GiB), below the 15,000,000,000-byte guard; the same byte limit separately
bounded each process's address space. Sampling is not an atomic aggregate memory
cap. All owned processes were reaped, and all 44 frozen input/source bindings
remained unchanged. The earlier unused 1,800-second/30-GiB proposal was never run.
No numerical integration, FORM/C++ package compilation, retry or resource
escalation followed this capability attempt.

The reference decomposes combined U/F support, whereas the earlier native
full-orthant result used 699 representatives of F-support charts and retained
orders `[-2,-1,0]`. These are internal representation choices; equal chart counts
or a nine-row native estimator are not required for end-to-end comparison through
epsilon zero. The separate native certificate proves exact whole-input zero
through order minus three. Below minus three, no representative contributes a
nonzero coefficient; at minus three, any surviving terms are parameter-independent
and their native exact sum is zero. It does not prove that each chart's minus-three
constant is individually zero.

A numerical comparison can retain the common three orders and their complete
native covariance, and check the provider's lower rows against that separate
exact-zero statement. Preserve both original artifacts and all provider rows;
do not manufacture Monte Carlo observations or pad the historical covariance.
The unmodified union-comparison API still reports `MissingEstimate` for absent
numerical rows, which must not be mistaken for missing scientific scope once the
certificate is considered. Current observations remain unmatched in revision and
timing boundaries, and no Pathfinder integration has yet been measured.

Retained evidence is under
`output/diagnostics/remaining-pathfinder-hard-four-loop-1/`: `assessment.json`,
the original invocation and frozen bindings, and `run/` with native stage timings,
input/strict-admission records, complete bundle, stack samples, logs and process
postflight. The stack samples show explicit derivative and evaluator construction;
they are observations of work, not estimates of uncompleted progress.
Independent retained-data review (`independent-review.json` in that directory)
accepts all 44 source bindings, exact input identity, complete order/sector counts,
stage timings and cleanup, without importing or evaluating the bundle again.
