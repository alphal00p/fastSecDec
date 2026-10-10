# Numerical dynamic Ward checks at 400 GeV

2026-10-10. Both incoming-gluon replacements now pass a fresh numerical
full-amplitude Ward check with each dynamic construction. This extends the
[complete amplitude control](contour-gghh-dynamic-amplitude.md), whose Ward
references were analytic only. The new gate uses the same explicit
S=0.8, L=1e-5, R=1 prescription. The previously inaccurate coarse default-L=1
box result remains unresolved.

## Native preparation and saved-artifact reuse

The existing native exporter keeps all polarization Gram products as runtime
inputs. Only the incoming gluon self-products are fixed to exact zero during
generation. Thus these saved artifacts can be rebound to either Ward state
without regenerating their mathematical programs. The colour projection,
model values, momenta, incoming routing and complete diagram catalogue remain
unchanged.

The preparation probe uses `FourMomentum<Atom>::dot`, with the original exact
native physical vectors and exact rational representations of the supplied
binary64 polarization components. It replaces epsilon1 by P(0), or epsilon2
by P(1), and exports the resulting Gram values through native evaluation.
There is no hand-written Lorentz product or replacement of a numerical
integration failure by zero.

For every diagram and both replacements, a second native route replaces the
actual polarization function in `FeynmanDiagram::projector` before contraction,
using the same operation as the existing HEPKit Ward reference. Both routes
then use `GraphIntegral`, native `SingleTerm` family preparation and Gaussian
numerator parameterization. Symbolica `expand` and `cancel` prove exact equality
of the complete parameter densities in all **16/16** cases. Their common
external-state-independent measure multiplier is unchanged. This proof took
9.250 seconds and 37.594 MiB sampled peak RSS.

The two triangle densities become literal zero through that native proof.
The six box densities remain nonzero, with different native term counts in
some equivalent representations. The actual saved triangle evaluators were
still run, and returned zero. No scalar-master reference is used as a substitute
for any of the numerical diagram contributions.

The maintained `gghh_one_loop_ward` example provides separate `prepare` and
`summarize` actions. Its preparation records the replaced incoming index,
consistent diagnostic vector components and complete exact Gram products. Its
summary calls the existing amplitude driver's `independent_sum` and `scaled`
helpers; the only change to that existing driver is helper visibility. It
requires all eight distinct manifest rows, independent seeds, matching native
reference leg/catalogue, full scope and complete durable native results.

## Predeclared numerical protocol

The ignored plan preceded sampling in
`target/contour-gghh400-dynamic-ward/plan.json`. The four runs are the Cartesian
product of two incoming replacements and polynomial/sign-aware construction.
Each uses eight separately seeded diagram integrations, with
`seed = 178139 + 104729*diagram_index + 1000003*(ward_number-1)`.
The two constructions use the same designs for comparison, but their estimates
are never pooled. Ward legs use different seeds.

Every newly bound artifact performs an actual eight-point contour pilot per
required source chart, followed by unchecked production. QMC starts at 1024
points and eight shifts, native Kuo33002 and Korobov3, with four caller-owned
workers and batches of 256. At most four allocations are allowed. The finite
coefficient's native absolute tolerance is 1e-3, with relative tolerance zero.
There is no relative-to-zero stopping rule. A non-target-reaching or failed
case would be retained and fail acceptance; all 32 cases reached their targets.

The coherent finite joint standard error must be at most `sqrt(8)*1e-3` in
native units. Every retained Laurent component and every individual diagram
is compared using five sampling standard errors plus the unchanged native Ward
reference allowance. That allowance is `1e-9 * 839.5530241054344`, or
`8.395530241054345e-7` in native units. It is distinct from the sampling error,
and is retained even for tiny pole residuals whose numerical roundoff need not
be described by the sampling covariance alone.

The existing reference records are reused without rerunning reduction or
OneLOop. All output vectors retain their full covariance. Independent diagram
covariances are summed with native DoubleFloat bookkeeping, then scaled once
by `(16*pi^2)^-2`; means are scaled by `1/(16*pi^2)`. No polarization-energy
normalization, colour average, extra rational term or inter-diagram multiplicity
is introduced. These Ward amplitudes carry the momentum replacement's units;
they are not physical helicity-amplitude estimates.

The total numerical/summary budget was 300 seconds and 3 GiB, with 30 seconds
per integration. All 32 integrations and four summaries exited successfully
in **59.179 seconds**, with **54.121 MiB** maximum sampled process-group RSS.
No budget limit fired. The reused copied CLI is the gated debug/SymJIT executable
with SHA-256 `84c4afceaaabccdc2231d7411dcbda5f1159f8577e0c7ffaf97c878a1b394a70`,
whose source/dependency provenance is in the amplitude review. These are
scientific controls, not release timing comparisons.

## Results

For each leg, the two independently generated construction artifacts yield
identical serialized mean vectors, complete covariances and allocation choices.
The table describes each construction separately; identical results are not
independent evidence or a variance-gain measurement.

| Replaced vector | Constructions | Native finite Re | Native finite Im | Joint standard error | Euclidean residual / joint error |
|---|---|---:|---:|---:|---:|
| epsilon1 → P(0) | Polynomial; sign-aware | 0.0015514335436144222 | 0.0007760860903413118 | 0.0011524923639391674 | 1.50519 |
| epsilon2 → P(1) | Polynomial; sign-aware | -0.0006105375045244443 | 0.00011985406508909452 | 0.0007696713478360923 | 0.808385 |

Every per-diagram native-reference comparison, full-vector zero comparison,
finite-precision gate and target-reaching gate passes. The physical-normalized
finite values are `(9.82456768634e-6 + 4.91462258011e-6 i)` for the first leg and
`(-3.86627391353e-6 + 7.58984733698e-7 i)` for the second. These residuals are
sampling estimates compatible with zero, not assigned exact zeros.

The complete native mean vectors, ordered `[-1 Re, -1 Im, 0 Re, 0 Im]`, are:

```text
Ward 1 mean = [-7.488519155922921e-18, -1.320830144237409e-17,
                1.551433543614422e-3,   7.760860903413118e-4]
Ward 1 SE   = [ 1.197695237700260e-17,  1.066555135024237e-17,
                8.279523245100726e-4,   8.017066778295272e-4]
Ward 2 mean = [ 9.187538239975529e-17,  1.136551732427673e-16,
               -6.105375045244443e-4,   1.198540650890945e-4]
Ward 2 SE   = [ 2.083249832902713e-17,  2.145633258123271e-17,
                5.345998373158302e-4,   5.537120168659107e-4]
```

Finite covariance blocks are
`[[6.855050516616326e-7,3.406675924233032e-8],
[3.406675924233032e-8,6.427335972764573e-7]]` and
`[[2.857969860581122e-7,2.159145177094849e-8],
[2.159145177094849e-8,3.065969976217146e-7]]` respectively. The full 4×4
covariances, including pole/finite correlations, remain in every native summary.

Per construction, the first leg evaluated 2,211,840 points across allocations
with 328,272 DoubleFloat rescues; the second evaluated 2,473,984 with 317,737
rescues. These are work counts, not final-estimator sample counts. There were
zero numerical failures, arbitrary-precision points or production causal-check
arguments. The final largest lattices had 8192 points per shift in four boxes;
the triangles used 1024, and FK08/FK10 used 2048 for leg one and 4096 for leg two.

## Reproduction and evidence

Build the maintained native example separately from any generation/integration:

```sh
nix-shell --run 'cargo build --locked -p fastsecdec --example gghh_one_loop_ward'
target/debug/examples/gghh_one_loop_ward prepare ORIGINAL_INPUTS FRESH_WARD_INPUTS
```

For each original diagram artifact and its new Ward card, run the ordinary CLI
with the settings above, saving a native result and checkpoint to a fresh
construction/leg work directory. Copy the corresponding prepared `manifest.json`
into that work directory. Then execute:

```sh
target/debug/examples/gghh_one_loop_ward summarize WORK_DIRECTORY \
  ORIGINAL_REFERENCE/hepkit/ward1/result.json FRESH_SUMMARY.json
```

Use `ward2` for the other reference. No operation starts sampling implicitly.
The ignored orchestrator preserves the exact 32 CLI argument lists and all raw
results under `target/contour-gghh400-dynamic-ward/`; the native preparation probe
and its completed exact proof are in `target/contour-gghh-ward-preparation/`.

| Evidence | SHA-256 |
|---|---|
| Ward plan | `7c55b6cad31a2de2ff555bc206b75ac9707bbc54ef98d95c7883c94717154c0e` |
| Numerical budget/results ledger | `2b9c0922dd895580e1825b7d960691c0c5e31cd16dd3c5029479dbf1a44e9bb3` |
| Numerical orchestrator | `995b5d7254d077b2ea39d409af1f859a7e01f091ebc482e2b306ade6edf09ecd` |
| Executed native preparation probe | `7f4684bc2af46ffa3a999fd7b35d3a4aedfc8c6e25f327fe081caf6611a58085` |
| Native exact equality proof | `0926b5df877510b6814380b7e6270017e4add1bc177f51544650c27acca62a7b` |

The maintained reproducer adds input/manifest guards and truthful replaced-vector
metadata to that executed native probe. The workspace all-target check passed;
a separate direct-rustc build against the pinned native libraries then repeated
all 16 exact proofs in 9.411 seconds, peaking at 37.805 MiB. Its complete proof
file has the same digest as the original probe, all 16 numerical point cards
are byte-identical, and all four summaries reproduce every numerical estimate,
covariance and acceptance field exactly. Updated diagnostic polarization-vector
metadata correctly describes the replacements. The replay/source-hash ledger is
`target/contour-gghh-ward-maintained/replay-result.json`, SHA-256
`8c3eb683781671bf521369e89928c6ab77a769dbf4d0c2c886f8c70db4503f58`.

Independent source review accepted the native substitution, exact density proof,
normalization and complete-covariance reuse. Its requested robustness checks are
included: the finite output must exist, and the inherited tolerance and
cancellation scale must each be finite and nonnegative. These replay/review
checks do not overwrite the original bounded-run provenance. The default-cap
discrepancy, multiloop physical checks and repeated independent amplitude
campaigns remain separate open gates.
