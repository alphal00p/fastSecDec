# Bounded dynamic ggHH checks at 400 GeV

2026-10-10. The bounded executions below now include the first native dynamic
triangle and box at 400 GeV. All causal checks passed, but the default-cap box
failed its numerical comparison at the initial work allocation. Smaller-cap
controls are separately recorded. This is not a completed dynamic physical
acceptance gate or evidence of a speed improvement.

## Reuse the existing physical input

The existing native exporter has already produced the exact common point at
`sqrt(s)=400 GeV`, `mH=125 GeV`, `mt=172.5 GeV`, `cos(theta)=4/5`, incoming `++`.
Use these original cards and runtime parameter files without rewriting their
graphs, numerators, helicities or measure:

* Triangle: `target/contour-gghh400-fixed/fastsecdec/inputs/diagram_00_triangle/`.
* Box: `target/contour-gghh400-fixed/fastsecdec/inputs/diagram_02_box/`.

The native HEPKit/OneLOop per-diagram references are
`target/contour-gghh400-fixed/hepkit/result/FK01.json` and `FK05.json`.
Their `native_finite_simple_double_pole` array is ordered **finite, simple pole,
double pole**, unlike the ascending Laurent layout of a FastSecDec estimate.

| Native coefficient | Triangle FK01 | Box FK05 |
| --- | --- | --- |
| epsilon^-2 | 0 | 0 |
| epsilon^-1 | 0 | -0.7278281615698874 |
| epsilon^0 | 0.7336482770235234 + 0.21195113867134305 i | 6.550371866775194 - 1.3247910833734284 i |

These are the native diagram values, before the physical `1/(16*pi^2)`
amplitude multiplier. They already use the same colour projection, model,
numerator and measure as the input cards. The measure is
`prod d^D k/(i*pi^(D/2))`, propagators `q^2-m^2+i0`, with the existing multiplier
`gamma(1-2*eps)/(gamma(1+eps)*gamma(1-eps)^2)`. In particular, the box's simple
pole is not zero; pole cancellation is a later complete-amplitude gate.

The full eight-diagram comparison and Ward evidence are retained in
[the fixed physical review](contour-gghh400.md). A one-diagram result must not
be compared against the complete-amplitude MadLoop number. The existing
`gghh_one_loop_me summarize` deliberately requires all eight diagrams and is
not an appropriate adapter for this pilot.

Both selected cards preserve native `family_preparation.SingleTerm` with
`max_states=32`. They declare physical products symbolically and supply their
values only through `point.toml`. No manual propagator removal or new graph,
reduction, polarization or CAS implementation is needed.

## Input identity

Both cards' SHA-256 is
`c288d9a7cdfbf23418f6bf42b494dca1965ff3fd9fbfa9b15cb3502a1166ed94`.
Selected files have these SHA-256 values:

| File | SHA-256 |
| --- | --- |
| Triangle graph.dot | `95a040d38f1b07dcd6ba74aa0a459739c3e0a41befcc91783edcb27042da5635` |
| Triangle point.toml | `79f6a4cb657eedada7f67140113ef42f5c8520a7a702e4796103b2976e9f095d` |
| Box graph.dot | `e2ced9e5aac012e6a3a67eae50561d142294f299050b572da392b3e58b0dcb61` |
| Box point.toml | `0d536a84b904f5fb891a24d4e801976e5abccf303be3df3a6fb3f1070ba07a50` |
| Either model.json | `789fe797dcc0fd6ae061137113865d18f4db4c802ba1473ba0c79eb52faf291d` |
| Either parameters.json | `41c0bdb34e4c22a88ff55e6b72b859adcd5e7c400ed06811945056b83a4d2da6` |

The independent reference graph BLAKE3 values are
`1d1bc32664593e5c811da0b21f2d26c9450187c2061d95e98fe9b7bf5d55fc5b`
for FK01 and
`02ba1eace574ee3739d78ac4a62b615f08e3857b89ca5988688700f4f61034bc`
for FK05. These are different hash algorithms, not mismatching identities.

If the ignored working inputs are unavailable, reconstruct them into a fresh
directory with the existing native exporter:

```sh
target/debug/examples/gghh_one_loop_me FRESH_INPUTS --sqrt-s 400
```

The source is `example/gg_hh_one_loop_ME/fastsecdec/exporter.rs`. Rebuilding or
regenerating is a separately scheduled action; the existing release executable
and unrelated long-running jobs remain untouched.

## First bounded execution

Use a newly built and gated debug CLI containing the explicit recipe flags.
Record its hash, source revision, dependency revisions, complete arguments and
peak RSS. `generate --contour` still denotes the fixed singleton; select the
dynamic recipe explicitly. The first stage is one polynomial triangle, one
worker, an eight-point pilot per sector, and one small production allocation:

```sh
fsd_bin="$PWD/target/debug/fastsecdec"
input_dir="$PWD/target/contour-gghh400-fixed/fastsecdec/inputs/diagram_00_triangle"
out_dir="$PWD/target/contour-gghh400-dynamic-triangle-pilot"
mkdir "$out_dir"

timeout --signal=INT --kill-after=10s 300s "$fsd_bin" --plain --json \
  generate "$input_dir/run.toml" --recipe dynamic-polynomial-v1 \
  --serial --workers 1 --output "$out_dir/integral.fsd" \
  > "$out_dir/generate.json" 2> "$out_dir/generate.stderr"

timeout --signal=INT --kill-after=10s 180s "$fsd_bin" --plain --json \
  integrate "$out_dir/integral.fsd" --parameters "$input_dir/point.toml" \
  --contour dynamical=0.8 --contour-construction polynomial \
  --lambda-cap 1 --displacement-cap 1 \
  --contour-validation always --contour-pilot-points 8 \
  --points 1024 --shifts 4 --workers 1 --seed 78139 \
  --absolute-tolerance 0 --relative-tolerance 0 --target-order 0 --max-rounds 1 \
  --save-result "$out_dir/result.json" --checkpoint "$out_dir/checkpoint.json" \
  > "$out_dir/integrate.json" 2> "$out_dir/integrate.stderr"
```

Run generation and integration as separate inspected actions; do not continue
after a failed or interrupted generation. `mkdir` must fail on an existing
output. The timeouts bound elapsed work and send ordinary cancellation before
forcing termination; they are not a numerical success criterion. Inspect
aggregate RSS externally and stop if it exceeds the agreed local budget.
The zero requested tolerances and single allocation deliberately expose an
unconverged `WorkLimit` result. A complete accepted vector may still support a
coarse comparison; it must not be relabelled `TargetReached`.

Default `L=R=1` is part of this first diagnostic, not a presumption of efficient
physical sampling. Dimensionful gradients can be large, and the displacement
bound alone does not limit stationary-point Jacobians. Record unusually large
outputs, precision rescues, failures and time. The predeclared smaller caps are
`L=1e-6`, followed by `L=1e-5` if another diagnostic is justified. Use them only
if the default fails or its numerical conditioning/variance requires further
investigation, rather than because the deliberately limited run says
`WorkLimit`. Each is an explicit separate prescription, pilot and saved result,
not an automatic rescue or a silent adjustment of the first result. Actual
generation and integration commands share a 600-second total process budget
and a 15-GiB aggregate process-group RSS limit.

Only after the triangle admits the physical point, completes its pilot and
returns finite complete coefficients should the same bounded procedure be
applied to `diagram_02_box`, a fresh `...-box-pilot` output, and seed `287597`.
Do not launch all eight diagrams. Next compare the sign-aware recipe using
`--recipe dynamic-sign-aware-v1` and `--contour-construction sign_aware` in a
separate fresh result. No result is pooled across prescriptions or reruns.

## Assessment and later refinement

Read the authoritative full-scope vector from native
`results::read_result(...).contributions.total`, retaining its complete
`orders`, `components`, means and full covariance. Match coefficients by
`(order,component)` against the per-diagram references above. The HEPKit files
do not publish uncertainties; do not invent exact reference uncertainty or
claim a certified sigma score. Use the existing native reference adapter if a
versioned comparison document is prepared, marking unreported uncertainty
`Unknown` and recording normalization/point evidence.

The accepted fixed baselines are:

* Triangle: `target/contour-gghh400-fixed/fastsecdec/work/diagram_00_triangle.result.json`.
* Box: `target/contour-gghh400-fixed/fastsecdec/refined/work/diagram_02_box.result.json`.

Both say `TargetReached`. The initial box under `fastsecdec/work` says
`WorkLimit` and is not the accepted refined result. Fixed triangle finite is
`0.7336485069901225 + 0.21195101244891681 i`; refined box finite is
`6.550132209680846 - 1.3243622467839697 i`. Their full covariance and pole
components remain in the native files.

A successful bounded pilot is only the first physical diagnostic. Accuracy
refinement, fixed/dynamic agreement with complete vectors, repeated-seed
matched-coordinate variance/timing, both validation policies, full-amplitude
pole cancellation and Ward checks remain later gates. No physical dynamic
performance improvement or three-loop readiness follows from this preparation.

## Actual bounded execution

The private copied debug driver has SHA-256
`3ce3043fc9e95ef66d0dd4825b69306c126ce67a5a2999a698d14edbedb825c9`.
It was built from the frozen working increment after
`c1f931ae3f1966b1668473f2d9e55d3fa3ada00f`, with Symbolica/Numerica
`516beb37d31af8e3d6ee321a7070f407a0b1b42d` and SymJIT
`d74993ffd76a6fc322a7bcf3963fa786783a38a8`. Native default execution uses
SymJIT O2; generation is symbolic/Taylor with the existing direct-translation
settings. This is a debug-profile CLI measurement, not release-driver timing.
The existing release executable was not replaced.

The ignored reproduction directory is
`target/contour-gghh400-dynamic-bounded/`. It retains the copied executable,
input/reference hashes, universal artifacts, native result/checkpoint JSON,
raw CLI reports, and each exact command with elapsed time and sampled aggregate
process-group RSS. `budget.json` sums process time across commands; `watch.py`
enforces the shared 600-second/15-GiB bounds. No source input was modified.

Serial one-worker generation completed for polynomial FK01 in 3.150 seconds
(71,610,368-byte observed aggregate peak), polynomial FK05 in 23.351 seconds
(160,108,544 bytes), and sign-aware FK01 in 2.932 seconds (82,325,504 bytes).
They contain three two-dimensional triangle sectors or four three-dimensional
box sectors. These small fixtures do not establish asymptotic memory scaling.

Every initial integration uses one worker, `S=0.8`, `R=1`, eight pilot points per
chart, 1024 points and four QMC shifts, one allocation, and `always` validation.
The requested tolerances are zero, so every completed initial run correctly
reports `WorkLimit`, not convergence. The first attempted 256-point request was
rejected before pilot evaluation because native Kuo33002 requires at least
1024 points; its error report remains retained. Only the point count changed
for the admitted run.

The complete polynomial estimates are below. Parentheses give the separately
reported real and imaginary standard errors; all cross-component covariance
entries remain in the saved native results.

| Diagram and cap | epsilon^-1 estimate | epsilon^0 estimate | Seconds |
| --- | --- | --- | ---: |
| FK01, L=1 | no pole coefficient | 0.190545205 + 0.876121640 i (0.384296, 0.466253) | 5.119 |
| FK01, L=1e-6 | no pole coefficient | 0.747031974 + 0.213079493 i (0.0202286, 0.0164534) | 5.192 |
| FK05, L=1 | -0.530442433 - 0.824468179 i (0.0292401, 0.0433308) | 5.085566906 + 7.902916902 i (0.237872, 0.488183) | 7.059 |
| FK05, L=1e-6 | -0.727826981 + 2.02991e-7 i (6.93567e-6, 1.03302e-6) | 5.599074130 - 0.987748133 i (0.364172, 0.754900) | 7.048 |
| FK05, L=1e-5 | -0.727823380 + 5.73182e-6 i (7.92792e-6, 1.15318e-5) | 6.546819688 - 1.329542967 i (0.00699294, 0.00535912) | 7.071 |

Both sign-aware triangle runs produced bit-identical complete mean/covariance
vectors to polynomial at the corresponding cap, taking 4.998 and 5.190 seconds.
This is an observed consistency control. It is not a paired variance study:
actual coordinate digests were not separately collected for these CLI runs.

All triangle pilots completed 24 points and 72 certified arguments. Box pilots
completed 32 points/arguments. Each triangle allocation executed 12,288 sector
samples and each box allocation 16,384. Production validation counts include
actual requests and retries; certificate precision reached 96 bits. Numerical
rescues reached DoubleFloat (106 bits), with no arbitrary-precision points,
nonfinite failures or cutoff zeros in these initial runs. Triangle rescues were
265 at L=1 and 270 at L=1e-6; box rescues were 193, 316 and 280 at the three caps.
Integration aggregate RSS peaks remained below 52 MB.

**The default FK05 numerical comparison fails materially**, even though all
causal checks pass. Its imaginary pole and finite coefficient differ from the
reference by much more than the reported sampling errors. Very thin regions
with large Jacobians are a possible explanation, not an established diagnosis.
Smaller-cap agreement cannot by itself prove default-cap correctness. This
requires independent multidimensional Jacobian/volume controls and adequate
sampling before accepting the default physical result. The full default
vector/covariance is preserved; it is neither discarded nor relabelled.

The L=1e-5 box gives a useful coarse agreement of both pole and finite vector,
but its initial joint finite uncertainty is still about 1.3 per mil. A separate
bounded refinement with the same cap/seed, target `1e-3` in the last complex
coefficient and at most four allocations is recorded independently. It does
not pool results from different caps or select a cap separately for each seed.

That separate FK05 refinement reached native `TargetReached` after three
allocations, with points 1024, 2048 and 4096, four shifts, Korobov3 and the same
seed `287597`. It returned:

| Coefficient | Mean | Standard errors (real, imaginary) |
| --- | --- | --- |
| epsilon^-1 | -0.7278282391396836 - 1.1016557975082537e-8 i | 8.995120812293881e-8, 8.089152395314169e-8 |
| epsilon^0 | 6.550315212806908 - 1.3247528628012906 i | 4.4254936729463636e-5, 5.584117389881257e-5 |

The native last-complex-coefficient relative uncertainty is `1.06617e-5`, below
the requested `1e-3`. Its full four-by-four covariance remains in
`box-polynomial-L1e-5-accuracy.result.json`; no reference uncertainty was
invented. The native reference differs by approximately 1.28 and 0.68 of the
reported finite real/imaginary standard errors, respectively. This is a useful
physical feasibility result at the explicitly selected small cap, not evidence
that the default cap is accurate at its previous allocation.

The refinement took 45.402 seconds, peaked at 51,908,608 bytes aggregate RSS,
and evaluated 114,688 sector samples. It recorded 115,164 production checking
arguments, maximum certificate precision 96 bits, 476 DoubleFloat rescues,
and zero failures or cutoff zeros. Including generation, all initial runs and
the rejected 256-point request, the cumulative process time was 116.733 seconds
and the maximum observed aggregate RSS was 160,108,544 bytes. Neither resource
limit fired. No additional diagram or full-amplitude execution was performed.
