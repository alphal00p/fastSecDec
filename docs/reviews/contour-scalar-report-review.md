# Independent scalar performance and report review

Accepted, 2026-10-10. This is a read-only comparison of the frozen native
campaign, the [curated results](../contour-scalar-results.json), the scalar
tables/plot scripts and the [report](../contour-deformation.typ). No generation,
sampling, reference evaluation or replacement estimator was run for this review.

## Evidence and identities

All 390 SHA-pinned inputs and raw records match, including the original
pre-execution design. An intermediate post-run documentation edit changed that
design's digest; the coordinator restored its exact `210e34e` bytes before this
final review. No campaign pin or numerical record was rewritten. The reviewed
curated JSON SHA256 is
`5f86980aef5d0507e3c52eb2b61f1fd2c17a0c6d391ee457823f3a2afcc9848c`.

The review checks all 24 generated owners, their closed process outcomes,
artifact sizes, saved identities, native generation modes, compiler settings,
reference records and actual residual schemas. All use Symbolic endpoint IBP,
native named coefficient expansion at initial relative width two, SymJIT O2,
Horner zero, CPE cap 1000 and one compiler core. The strategy is in-process
caller-driven generation, not the CLI serial coordinator.

All 144 complete native epochs and 72 matched Jacobian pairs are accounted for.
Full Laurent vectors and covariance matrices are copied unchanged from native
results, including exact offsets once. Actual coordinate/weight hashes and
settings agree between Jacobian choices. The maximum scaled complete-vector
mean difference is `1.4673721389269612e-16`. Raw native semantic decoding was
not repeated; the executed native restore/admission checks and their byte pins
remain the ownership boundary.

The independent reproduction of these checks is retained under
`target/contour-scalar-report/generation-independent-review.{py,json,log}`;
the frozen measurements are under `target/contour-scalar-runtime/campaign1/`.

## Timing, memory and program sizes

The generation table correctly sums `native_generation_seconds` and
`compilation_seconds`. It excludes fixture/reference construction and saving.
Generation RSS covers the entire process, including those excluded stages.
Consequently the one-loop peaks, which include OneLOop reference work, must
not be interpreted as isolated compiler memory. The report states this scope.
Exact IR bytes and lowered SymJIT IR bytes are separate sums over actual
stochastic programs, not whole saved-owner size.

Sampling time subtracts only measured coordinate hashing. Context creation,
precision-cache warm-up and caller work remain included. The displayed
evaluator average is the mean of per-sector means over the two final-epoch
runs; the maximum is the largest sector mean. Each sector numerator sums
native f64, double-double, arbitrary and conditioning timing, and the denominator
is its actual accepted point count. Precision retries are therefore included.
This quantity is neither maximum point latency nor a full runtime amortization.

The original sunrise processes had unobserved sampled generation RSS, rather
than measured zero. The six separately labelled GNU-time follow-ups retain
unchanged native owners and low-memory launcher observations, giving roughly
59.8 MB native high-water readings. Their timings do not replace the original
generation times. Other rows retain sampled peaks, which may miss short spikes.

## References and active deformation scope

The prepared references agree with every saved owner and runtime reference
vector. C0/D0 use the native OneLOop expression backend and explicit `1/r_Gamma`
normalization. The sunrise retains its `-1/4` pole and the lower-lip finite
imaginary part `-pi/2`. The kite retains the exact five native denominators,
U/F and explicit measure multiplier `-1`; its native zero-regulator reference
is `4.403658192582334 + 1.5704037847171362 i`. The previously reviewed finite
regulator in the upstream printed value is separate from the series truncation
bound and from binary64 roundoff.

Actual surviving first-image input slots in the Dual build observations are:

| Case | Fixed | Polynomial | Sign-aware |
| --- | ---: | ---: | ---: |
| Triangle, two residual sectors | 4 | 8 | 8 |
| Box, two residual sectors | 5 | 14 | 14 |
| Sunrise, one residual sector | 0 | 0 | 0 |
| Kite, eleven residual sectors | 57 | 69 | 68 |

These are sums of compiled input slots, not callback/sample counts. The
sunrise's constant residual causal factor makes the deformation inactive;
its six agreeing outputs are a branch/subtraction control. The one-loop
quadratic causal F and linear positive U have no higher odd/even envelope
terms, respectively, so polynomial and sign-aware radius equations coincide.
The matching numerical vectors support that interpretation. It is not a claim
that the two constructions coincide for the kite or other topologies.

## Performance interpretation

The reported final polynomial/fixed finite covariance-trace ratios reproduce
as 157.1310 for the triangle, 6.23765 for the box, 1 for the sunrise and
2.17724 for the kite. They describe the predeclared common-cap comparison;
all cases admit cap 0.1, but dynamic strength is bounded by `S*L=0.08` while
fixed strength is 0.1. They do not compare individually optimized prescriptions.

For sign-aware kite generation, the measured Symbolic/Dual times are
4.021874/12.297302 seconds. Mean evaluator costs are 10.199370/8.557693
microseconds per accepted sector-point, including precision rescue. Dual is
about 3.058 times slower to generate and about 16.1% cheaper to evaluate in
this observation. This supports the stated tradeoff, not a universal generation
or end-to-end speedup. The exact/JIT program sizes and separate memory rows
remain necessary context.

The convergence plots average the two native standard-error observations at
each lattice size and show their span; they do not pool estimates/covariance.
The endpoint slopes, including the triangle's 8.39, are descriptive three-size
observations rather than established asymptotic rates. The maximum final finite
reference-error norm divided by native joint standard error is 1.87597. This
does not calibrate uncertainty coverage with two seeds. All recorded epochs
retain zero numerical failures, unstable/cutoff-zero outcomes, arbitrary-precision
rescues and optional production causal checks; 50,660 double-double rescue
observations across the separate epochs remain included in cost.

No report correction remains from this review. The complete scalar suite and
the physical D05 results do not complete the broader Phase B programme.
