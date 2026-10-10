# Bounded full dynamic ggHH amplitude at 400 GeV

2026-10-10. This extends the [single-diagram dynamic controls](contour-dynamic-physical-readiness.md)
to the complete two-triangle/six-box top-loop `++` amplitude. The explicit
small-cap prescription is tested; the previously inaccurate coarse default-cap
box result remains unresolved and is not replaced by this result.

## Native inputs, references and covariance ownership

The existing native exporter already supplied all eight labelled diagrams at
`sqrt(s)=400 GeV`, `mH=125 GeV`, `mt=172.5 GeV`, `cos(theta)=4/5`, zero widths,
and the common incoming `++` helicity convention. This run reuses
`target/contour-gghh400-fixed/fastsecdec/inputs/` without modifying graph,
model, numerator, runtime point or normalization. The manifest identifies the
complete native generator catalogue and retains distinct diagram seeds
`78139 + 104729*i`, for i=0,…,7. No diagram multiplicity is inferred by hand.

The unchanged normalization is `M_ab = delta_ab A++`, with the native
`delta_ab/8` projection, measure `d^Dk/(i*pi^(D/2))`,
`gamma(1-2*eps)/(gamma(1+eps)*gamma(1-eps)^2)`, and one final physical multiplier
`1/(16*pi^2)`. There is no spin/colour average or separately added rational
term. Per-diagram native HEPKit/OneLOop references and the complete independent
MadLoop result are reused from the fixed 400 GeV reproduction; no reference
calculation is rerun. Their common amplitude is
approximately `-0.0194295209824 - 0.0117460318328 i`.

The existing Rust `fastsecdec/summarize.rs` and `compare.rs` are compiled
verbatim through ignored entry-point wrappers. They consume native versioned
result documents, validate full scope, complete target-reaching production and
manifest seeds, retain each full Laurent covariance, and sum independently
seeded diagram estimates with native DoubleFloat bookkeeping. They scale the
complete covariance by the square of the physical multiplier. A construction's
estimates are never pooled with the other construction or earlier scalar runs.
The existing five-standard-error Laurent pole checks, joint complex amplitude
comparison, and at-most-0.1% amplitude uncertainty gate remain unchanged.

Both existing native Ward-substitution reference records are checked by that
comparison driver. **This is not a new numerical dynamic Ward-substitution
integration**; that distinct gate remains pending. A new Ward source, if needed,
must reuse the existing HEPKit symbolic substitution and native point binding.

## Predeclared bounded protocol

The protocol was recorded before execution in
`target/contour-gghh400-dynamic-amplitude/plan.json`, together with 46 SHA-256
input/reference hashes. Both polynomial and sign-aware construction use
S=0.8, L=1e-5 and R=1; this is an explicit caller prescription, with no implicit
F normalization or changed default. Each diagram is freshly generated using
serial native generation with one worker. Each bound artifact receives eight
actual checked pilot points per source chart, then unchecked production.

QMC begins at 1024 points and eight randomized shifts, with native Kuo33002,
Korobov3, evaluation batches of 256, four caller-owned workers, the manifest
seed, zero absolute tolerance and relative tolerance `1e-4` on the finite complex
coefficient. At most four allocations are allowed. The two constructions run
separately; generation and integration are separate actions, and a failed
creation cannot be consumed. Resource limits are cumulative 600 seconds and
15 GiB aggregate process-group RSS, with 100 seconds per generation and 90 seconds
per integration. Ordinary cancellation precedes forced wall-time termination.
A WorkLimit or resource limit is retained as inconclusive, never relabelled
TargetReached or zero.

The tested copied CLI is
`target/contour-gghh400-dynamic-amplitude/fastsecdec`, SHA-256
`84c4afceaaabccdc2231d7411dcbda5f1159f8577e0c7ffaf97c878a1b394a70`.
It was built from `184803d` plus the current native diagnostics and family
working slice, after the CLI process gates passed. Dependencies are public
Symbolica/Numerica `516beb37d31af8e3d6ee321a7070f407a0b1b42d` and SymJIT
`d74993ffd76a6fc322a7bcf3963fa786783a38a8`. This is a debug-profile CLI with
native SymJIT kernels, not a release-driver performance comparison.

`run.py` only orchestrates commands; `watch.py` bounds and records each process
group. Ignored `runs/*/execution.json`, raw CLI output, native result/checkpoint
files and `budget.json` retain every allocation, command and limit outcome.

## Results

Both constructions completed all eight diagrams with `FullIntegral` scope,
`TargetReached`, complete production, and all eleven unchanged native comparison
checks passing. All 36 generation, integration, summary and comparison commands
exited successfully. Total measured subprocess time was 262.097 seconds and
maximum sampled aggregate RSS was 165.152 MiB; no resource limit fired.

The independently generated and run polynomial and sign-aware artifacts have
different content IDs. Their per-diagram Laurent means, complete covariances and
QMC allocations are identical at the serialized binary64 values in this
experiment. The following shared numbers describe each construction separately;
they are not a pooled estimate or independent replication between constructions.

| Construction | Physical Re A++ | Physical Im A++ | Joint standard error | Relative standard error | Distance to HEPKit / MadLoop |
|---|---:|---:|---:|---:|---:|
| Polynomial | -0.01942819199549045 | -0.011743160377985895 | 2.354196386749392e-6 | 0.01037024% | 1.344021 / 1.344021 standard errors |
| Sign-aware | -0.01942819199549045 | -0.011743160377985895 | 2.354196386749392e-6 | 0.01037024% | 1.344021 / 1.344021 standard errors |

The joint error is the square root of the Re/Im covariance trace. Reference
distance is the existing driver's Euclidean complex difference divided by that
joint error, rather than a Mahalanobis confidence statistic. The colour-summed
fixed-helicity square is `0.004122851679013897`, with propagated standard error
`6.276288892269621e-7`; its first-order propagation uses the Re/Im covariance.

The complete physical Laurent vector, ordered `[-1 Re, -1 Im, 0 Re, 0 Im]`, is

```text
mean = [-9.364745865256397e-8, -2.673414927530339e-7,
        -0.01942819199549045, -0.011743160377985895]
SE   = [ 7.538020679493478e-8,  1.948216643297963e-7,
         1.131733502558500e-6,  2.064320688887887e-6]

covariance of mean =
[[ 5.682175576447130e-15,  3.669516685660822e-15, -7.948587823962100e-14, -4.719708956913822e-14],
 [ 3.669516685660822e-15,  3.795548089223181e-14, -8.492887975439420e-14, -3.929583577858348e-13],
 [-7.948587823962100e-14, -8.492887975439420e-14,  1.280820720813330e-12,  1.024844040568609e-12],
 [-4.719708956913822e-14, -3.929583577858348e-13,  1.024844040568609e-12,  4.261419906570561e-12]]
```

The surviving simple-pole components are respectively 1.24 and 1.37 standard
errors from zero and pass the unchanged five-error gate. No divergent component
has been discarded when making the finite comparison. The native unscaled full
vector and every per-diagram covariance also remain in each result document.

| Diagram | Final points per sector and shift | Evaluated points across allocations | DoubleFloat rescues |
|---|---:|---:|---:|
| FK01 | 1024 | 24576 | 958 |
| FK02 | 1024 | 24576 | 1167 |
| FK05 | 4096 | 229376 | 998 |
| FK06 | 4096 | 229376 | 722 |
| FK07 | 4096 | 229376 | 879 |
| FK08 | 1024 | 32768 | 561 |
| FK09 | 4096 | 229376 | 875 |
| FK10 | 1024 | 32768 | 516 |

Each construction evaluated 1,032,192 points across allocations and recorded
6,676 native DoubleFloat rescues, zero failed or unstable points, and no
arbitrary-precision points. These evaluated-work totals include earlier
allocations; they are not the final estimator sample count. Eight actual pilot
points per required chart preceded production. Production causal-check counts
are zero, as prescribed by Pilot policy. The fixed seeds, full Laurent vectors,
and complete covariance are retained rather than selecting favourable components.

## Reproduction and independent review

The final ignored output root is
`target/contour-gghh400-dynamic-amplitude/`. SHA-256 anchors are:

| File relative to that root | SHA-256 |
|---|---|
| `plan.json` | `dd3cbf091d8df93e9dc630fedb49708bef757ba187288192afdf672c0924e7db` |
| `run.py` | `e5234d6d5276b8680227ae3438ea9ca46514a56d980a84e2135f81228572d4b0` |
| `watch.py` | `a52d6bf08f826a62c323f439bc63f799323dc9fb13e15a96fcf63f9cee5a6f4a` |
| `budget.json` | `39316c03c3e8c8928563ae5fd2c50cffe8e061b19913b48bd5a3ebcf2c0a2a16` |
| `polynomial/fastsecdec/result.json` | `5243df85f77c1e168a5865480892c457cfac05603cc409d2e0cb9d2cdf2ac87d` |
| `sign-aware/fastsecdec/result.json` | `693cec4ad8b2176af98aa72a63f6db95d4925e1ce1b67f756c871164c8e7db09` |
| Either `comparison.json` | `ed2ee2f6f0dd37485d2d7758f659be39cbe011f96d7dd770f227cc21d41fcb60` |

The independent generation reviewer checked the eight native manifest rows,
distinct seeds, separate construction directories, declared settings, unchanged
Rust summary/reference drivers, complete covariance propagation and exactly one
physical normalization. That source review found no blocker. Both full runs
then satisfied the required native summary and comparison gates.

This accepts the bounded full `++` amplitude at this point with the explicitly
declared L=1e-5 prescription. The coarse inaccurate default-L=1 FK05 result
remains unresolved. There is no claim of a default-cap convergence guarantee,
construction variance gain, release performance, independent seed replication,
new numerical dynamic Ward check, or completed broader physical acceptance.
