# Original hard four-loop Pathfinder comparison

The single bounded Pathfinder generation attempt completed the original
full-positive-orthant input and passed its strict prepared-bundle reader. The
subsequent fixed-work native/Pathfinder observation below also completes both
full numerical vectors. **Neither result reaches one per mille.** This is not
repeated timing acceptance or an instrumented sample-latency comparison.

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
certificate is considered. The historical generation observations remain
unmatched in revision and timing boundaries; the subsequent numerical comparison
records its actual allocations separately below.

Retained evidence is under
`output/diagnostics/remaining-pathfinder-hard-four-loop-1/`: `assessment.json`,
the original invocation and frozen bindings, and `run/` with native stage timings,
input/strict-admission records, complete bundle, stack samples, logs and process
postflight. The stack samples show explicit derivative and evaluator construction;
they are observations of work, not estimates of uncompleted progress.
Independent retained-data review (`independent-review.json` in that directory)
accepts all 44 source bindings, exact input identity, complete order/sector counts,
stage timings and cleanup, without importing or evaluating the bundle again.

## First complete fixed-work comparison, 2026-10-06

The accepted archived `e25ca59` native CLI generated the unchanged original card
in **12.976408 seconds**, including **2.393917 seconds** attributed to SymJIT O2
compilation. Strict inspection retained all 2,760 charts and 699 nine-dimensional
kernels. This fresh CPU0 generation had a 300+5-second/15-GB bound; it does not
replace or reinterpret the older 36.755-second observation. The existing
Pathfinder bundle was reused without regeneration.

Both numerical processes used CPUs0–7/eight workers, Korobov3, sixteen shifts,
seed `20261009`, full scope, normal precision and the ordinary O2 routes. Native
used published HKKN alpha3 at exactly 1,024 points per shift; Pathfinder used its
normal published prime catalogue, where the same request selects 1,123 points.
Pathfinder retained optimized boundary support and the correlated all-sector sum.
These are different rules and partitions, not an identical-rule timing pair.

| Observation | Native | Pathfinder |
| --- | ---: | ---: |
| Complete integration process, including loading | 68.735126 s | 279.688857 s |
| Numerical sectors | 699 | 3,728 |
| Actual complete-vector kernel points | 11,452,416 | 66,984,704 |
| Finite-part estimated relative uncertainty | 65.225‰ | 53.177‰ |
| Sampled process-tree peak RSS | 292,139,008 B | 2,649,841,664 B |

| Epsilon order | Native mean ± standard error | Pathfinder mean ± reported error |
| --- | ---: | ---: |
| −8 through −3 | Separate exact whole-input zero certificate; no numerical rows | Every row `0 ± 0` |
| −2 | −3.777457396222 ± 0.296930802742 | −3.626271847617 ± 0.165609320704 |
| −1 | −15.921446940760 ± 1.162753406498 | −16.926324682668 ± 0.925803224283 |
| 0 | −144.851060142433 ± 9.447910180715 | −153.621901556016 ± 8.169121310473 |

Native preserves its full three-by-three total covariance and all 699 marginal
covariances; the independent review recomputed the shared-shift total to within
`2.22e-16`. Pathfinder preserves all nine real and imaginary rows and its native
reported physical errors; every imaginary value/error is zero. No external
cross-order covariance is invented. The three available native reference checks
and four Pathfinder checks pass, with maximum absolute pulls 0.638 and 0.333.
All six lower Pathfinder rows separately agree with the whole-input zero
certificate. This does not turn their constituent sectors into exact zeros or
fill the native union reader's `MissingEstimate` at order −3. The three numerical
cross-provider differences are less than half the sum of their reported errors;
that descriptive comparison does not assume independence of these same-seed,
different-rule observations.

Native reports zero evaluation failures, 2,325,753 rescues, 1,184 weighted checks,
970 additional replays and maximum precision 384 bits. Pathfinder reports
62,432,991 ordinary samples and 3,758,665 / 543,211 / 249,837 samples in its normal
32 / 100 / 1,000-decimal rescue lanes. Its complete finite outputs and all inactive
nonfinite sector-error diagnostic tags remain in the raw transport.

Native average worker cost is 44.899 µs per point, with maximum sector-average
worker cost 133.637 µs. Pathfinder's evaluator bucket averages 7.848 µs per point
(11.876 µs including its Python bucket), with maximum sector-average evaluator
cost 120.260 µs. These instrumentation boundaries differ: native worker time
includes sampling, transformation, evaluation, replay and accumulation. They
do not establish isolated evaluator parity, corresponding-sector timing, or an
individual-sample maximum. No instrumented sample-latency run was added.

Each numerical side had a 600+5-second/15-GB bound. The first Pathfinder prefix
failed before importing its scientific code: the existing timer canonicalized
the Python symlink and bypassed the installed virtual environment. Its
**2.679500 seconds** remain recorded. Restoring the previously accepted
`prlimit`/`taskset` command wrapper preserved the exact virtual-environment path;
no package or scientific setting changed. The corrected attempt used only the
remaining **597.320 seconds** of normal budget. All original and corrected
processes were reaped; frozen source, input and bundle checks passed. A requested
read-only CPU snapshot landed after completion and records no live processes;
the completed timer retains 912.506 user and 37.786 system CPU seconds for
Pathfinder, without an extrapolated progress claim.

Evidence: `output/diagnostics/hard-four-loop-fixed-work-1/` contains generation,
the complete native result/checkpoint, the failed pre-import prefix and its
independent native result review. `hard-four-loop-fixed-work-2/` retains the
reviewed interpreter correction, the complete Pathfinder result, cleanup,
`assessment.json` and the final independent result review. These observations
close first full-vector execution and available-reference comparison; the
highest-order target and repeated representative performance acceptance remain
open. No ladder, repeat or further Pathfinder allocation followed this pair.
