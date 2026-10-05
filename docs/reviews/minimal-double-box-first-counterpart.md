# Bounded double-box first counterpart

On 2026-10-05, the first matching Pathfinder prepared integration completed
successfully. This is one fixed-work observation, not a repeated timing median,
convergence comparison, or final performance acceptance. Both implementations
miss the required one-per-mille reported error on the highest requested signed
epsilon order, zero. No further numerical level was launched.

The frozen plan remains
`output/benchmarks/minimal-paired-current-3/plan.json`, SHA-256
`b105dba6cb045f9dbe447551d0e176b9ad965e360843d33260eca07f682bb9c5`.
Both rows use 1,024 Kuo/QMCPy-linear points, sixteen shifts, seed 20261302,
Korobov3 and eight workers on CPUs 0–7. Their sector partitions and randomized
point dependence differ as described in [the execution review](minimal-paired-acceptance.md).
Native remains Symbolica 3.0.1 with reviewed local patches / SymJIT 2.26.4;
Pathfinder remains Symbolica 2.1.0 / embedded SymJIT 2.18.6. Both use O2 and
their existing enabled precision policies. Generation used one worker on CPU 0.

| Observed quantity | Native | Pathfinder |
| --- | ---: | ---: |
| Whole prepared-integration process, seconds | 30.404195 | 17.632585 |
| Internal numerical loop, seconds | 30.032925 | 17.036334 |
| Sectors | 102 | 96 |
| Actual accepted/evaluated rows | 1,671,168 | 1,572,864 |
| Finite coefficient | −15.61258032 | −15.15776078 |
| Reported finite error | 0.52370930 | 1.66651570 |
| Relative reported finite error | 3.3544% | 10.9945% |
| One-per-mille target observed | No | No |

The single whole-process native/Pathfinder ratio is 1.724319, exceeding the
plan's 5% timing allowance at this fixed work. This observation does not establish
the ratio at equal accuracy. Native performs 6.25% more physical rows. Native's
jointly estimated covariance and Pathfinder's reported L1-propagated prefactor
errors are different uncertainty quantities; no efficiency ratio is inferred
from their squared errors.

The complete returned real Laurent vectors are retained:

| Signed order | Native mean ± standard error | Pathfinder mean ± reported error |
| --- | ---: | ---: |
| −4 | 0.0006024030 ± 0.0011065581 | −0.0090296578 ± 0.0068973774 |
| −3 | 1.5040530291 ± 0.0110239566 | 1.4690055550 ± 0.0581781509 |
| −2 | 1.2898947408 ± 0.0287731797 | 1.2105210143 ± 0.2648547458 |
| −1 | 2.8979524033 ± 0.0842638618 | 2.6914428941 ± 0.6148277641 |
| 0 | −15.6125803226 ± 0.5237093032 | −15.1577607795 ± 1.6665156952 |

Both rows pass the existing comparison against all five available independent
real reference coefficients. Native retains its full 5×5 covariance of the mean
and marks production complete. Pathfinder completed all 96 sector groups with
sixteen estimates for every active coefficient. Its five imaginary estimates
and errors are zero, but the common reference has no imaginary coefficient rows:
the reference observer therefore reports full complex-union coverage false and
five `missing_reference` rows. No reference or estimate was padded. The native
reader's real-only union passes; these differently sized unions are not an
identical coverage claim. Pathfinder does not provide joint Laurent covariance.

Native records 766,169 rescues/conditioning checks, 229 weighted checks,
99 additional replays, maximum precision 512 bits, and zero evaluation failures.
Pathfinder records 1,329,841 ordinary rows, 199,130 stability rows at 32 digits,
29,923 medium-precision rows at 100 digits and 13,970 high-precision rows at
1,000 digits. It exposes no separate aggregate failure count in this report;
all admitted aggregate coefficients and errors are finite.

The native pooled accepted-worker cost is 142.982376 microseconds per row;
its largest sector mean is 837.212447 microseconds. Pathfinder's evaluator-only
and evaluator-plus-Python buckets are 55.398678 and 64.437718 microseconds per
row; its largest evaluator-only sector mean is 414.805365 microseconds. The
native interval includes point generation and weighted checks that the reference
buckets attribute elsewhere. These are attribution, not a matched JIT ratio.
An individual-sample maximum remains unmeasured for this case.

The old campaign driver PID 405604 and its completed child/timer were absent at
preflight, so the original stopped-PID guard could not run. The coordinator
approved `scheduling-2/reference_first.py`, which verifies either the expected
stopped command or its absence with no other same-user live campaign runner.
It preserves `scheduling-1` verbatim and changes no numerical command, limits,
prepared program or observer. The original script's actual internally bound
hash was `511de522635f7757ac4985896b5feb94effd2fbad26f4f38f92a39b6e0ec668a`;
the handoff's `cea32…` value was stale and is recorded as such, not certified.
The scheduling-2 script hash is
`d12ff97c2ef22e2281d104e00cb1ed195279752d962ab378de3a258430be0aae`.
All 617 frozen plan files, 375 prepared-program files, correction inputs and
formula read-cache inventories verified. Both implementers held scientific
runtime during the 17.632585-second process and 0.420749-second admission;
compilation on separate CPUs was permitted. Unrelated host workloads were left
unchanged. The scientific slot was released after both processes were reaped.

The existing native N=8,192 process/result remains retained but has no
`observation.json`; it is unadmitted and is not called accepted here. Neither
that driver nor a larger ladder was restarted. The on-shell triple-box campaign
remains closed under the user's accepted generation-only scope.

Machine-readable evidence is
`output/benchmarks/minimal-paired-current-3/scheduling-2/counterpart-summary.json`,
SHA-256 `8e16c8d07ac2b10feb6366e11cda998bcf21b6eca478ded49189a7e5a2c1c1b5`.
It binds the untouched plan, scheduling provenance, both complete result and
admission records, precision counts and native covariance. The new scientific
row is `double_box/integration-1-reference-n1024/` beneath the same campaign.
The coordinator independently reviewed the scheduling-only diff, all fourteen
summary bindings and covariance symmetry, and accepted this factual report.
This does not accept convergence or performance; those gaps stay open.
