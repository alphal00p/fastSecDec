# Remaining independent scientific references

This is a source-checked execution protocol; completed outcomes are recorded
separately in [the bounded-attempt record](remaining-reference-attempts.md).
It uses the ignored Pathfinder checkout at
`582d8c7f6dde9bf750750d4c2a2d85a94ce940cd` and its installed pySecDec 1.6.6.
No Python dependency or implementation is added to FastSecDec. Native HEPKit
one-loop masters and reduction do not supply these multiloop integrals; the
existing external pySecDec engine owns decomposition, subtraction, Gaussian
numerator integration, compiled evaluation and numerical integration here.

## Priority and bounded attempts

| Attempt | Scientific input | First independent route | Initial bound |
| --- | --- | --- | --- |
| 1 | Double-box complete Laurent vector through zero | Existing native pySecDec DOT CLI, original seven propagators | 600 s total, 30 GiB process-tree RSS |
| 2 | Issue 1, complete positive orthant through order two | Existing native pySecDec direct-U/F all-sector CLI | 600 s total, 30 GiB; numerical deadline 180 s |
| 3, 4 | Off-shell triple-box scalar and rank-two numerator | Native `LoopIntegralFromPropagators` with the exactly equivalent eight active denominators | Separate 600 s/30 GiB attempts; generation failure retained |
| 5 | Hard four-loop complete positive orthant through zero | Existing native pySecDec all-sector U/F CLI | 600 s total, 30 GiB; numerical deadline 180 s |

These are initial attempts, not promises that package generation or the requested
precision will fit. The coordinator assigns the exclusive symbolic runtime before
launch. Successful package generation permits a separately bounded numerical
refinement using the identical package; it does not authorize silently extending
the first attempt or replacing an inconvenient result. Every attempt retains its
command, source hashes, generated package, logs, exit status, elapsed time and
memory watchdog outcome. A failure is evidence of an unresolved reference gate.

For the first double-box attempt the prepared wrapper is
`output/probes/run_double_box_reference.sh`. It refuses to reuse an output
directory. Run it from a coordinated runtime slot as:

```sh
bash output/probes/run_double_box_reference.sh \
  /common/dev/fastsecdec/output/diagnostics/remaining-pysecdec/double-box-attempt-1
```

The first double-box attempt reached its 600-second bound in native FORM code
generation and produced no numerical result. Its preserved outcome is recorded
in [the bounded-attempt record](remaining-reference-attempts.md). Workspace
compilation overlapped with coordination; no elapsed time from this scientific
attempt is a matched performance measurement. The directory above now exists
and must not be reused or overwritten.

Two separately approved copied-package continuations preserved that failure.
The second reached its compile bound; the third completed the remaining native
make/link and returned all orders minus four through zero with nonzero measured
real uncertainties. Independent source, normalization and full-tuple review,
followed by native Rust comparison, supports the new
`examples/references/double_box.json` correctness fixture. Aggregate external
work and uncertainty calibration remain unknown. Issue 1 subsequently completed
both the initially Unverified disteval route and a separately audited ordinary
sector-sum observation. The next unexecuted route is the projected off-shell
scalar triple box, using the refined
[ordinary sector-sum protocol](projected-triple-reference-attempt.md).

Its ignored `capture_pysecdec_series.py` launcher observes only the argument to
the existing `_parse_pysecdec_json_series` function and delegates unchanged to
that function and `FSD.py`. Before parsing it closes an original-representation
file and a typed JSON encoding that preserves tuple dictionary keys and exact
binary64 values. This retains the complete native returned tuple even if the
legacy parser subsequently fails. The launcher hash is part of preparation;
there is no frozen-source patch or additional numerical calculation. A pure-data
sentinel-parser regression verifies persistence before a thrown parser error.

The wrapper records the exact argv and input/tool hashes before launching the
existing reference watchdog. It selects one allowed Linux CPU and fixes native
thread counts to one. The DOT bridge's `loop_package` call does not pass a
`processes` argument; installed pySecDec derives its default from CPU affinity.
Affinity therefore prevents the simultaneous Symbolica instances previously
observed in the self-energy reference. On macOS use the same native pySecDec API
with explicit `processes=1`; there is no portable `taskset` equivalent assumed.

## Double-box: direct command and normalization

In the frozen Pathfinder directory the scientific command is:

```sh
.venv/bin/python FSD.py run \
  --dot-file examples/graphs/double_box.dot \
  --kinematics examples/graphs/double_box_kinematics.yaml \
  --mode massless --dot-engine pysecdec \
  --prefactor-convention pysecdec --sector-method iterative \
  --workers 1 --max-eps-order 0 \
  --pysecdec-maxeval 65536 --pysecdec-epsrel 0.01 \
  --pysecdec-workdir "$REFERENCE_ATTEMPT/package" \
  --result-path "$REFERENCE_ATTEMPT/result.json" \
  --json --no-progress
```

The wrapper supplies the compiler/make environment, affinity, watchdog and the
absolute `REFERENCE_ATTEMPT` path. No historical target is attached. This is the
actual shipped topology, including its degree-two internal vertex and repeated
physical denominator; an ordinary undotted planar-ladder formula is not an
interchangeable reference. The scientific point is massless, all external
virtualities zero, `s12=s23=-1`, with normalized per-loop measure
`d^D k/(i*pi^(D/2))`, `D=4-2*eps`, and no additional scale/Euler factor.

`build_native_pysecdec_dot_bundle` and `run_pysecdec_package` in
`src/pysecdec_bridge.py` skip Pathfinder's sector evaluators and use native
`LoopIntegralFromGraph`, `loop_package` and `IntegralLibrary`. The generated
package includes the physical Gamma factor. Check the generated package's
prefactor code and metadata before accepting a pole vector: the installed public
`pySecDec.make_package` is a one-integral **sum-package wrapper**, whose weighted
integral already multiplies its constituent prefactor. Consequently the frozen
parser's tuple-entry-zero extraction can include the physical prefactor even
though the generic `IntegralLibrary` tuple calls that entry "without prefactor".
This was confirmed in the existing kite package's generated weighted-integral
and `prefactor.cpp` sources. Retain the same evidence for each new package; do not
manually convolve another Gamma factor or assume the generic tuple label removes
it.

`maxeval=65536` is a steering parameter, not an observed sample count. This bridge
does not expose the actual QMC lattice sizes/counts or its generated random seed.
Installed `IntegralLibrary` defaults to QMC with `minn=10000`, seed zero/native
random initialization and an absolute-error target of `1e-7`; the bridge changes
only relative tolerance and maximum-evaluation steering. Its `--seed` field is
not evidence that the native C++ QMC used that seed. Retain native logs, and label
unobserved counts/seed as unavailable. A later direct `IntegralLibrary` refinement
may explicitly configure native `use_Qmc(seed=..., verbosity=...)` and preserve
the complete returned tuple; that is reference orchestration, not a new sampler.

The exact native proof that the leading order minus four vanishes remains an
independent algebraic check. The new independently audited numerical fixture now
covers orders minus four through zero; its measured leading coefficient and
error are retained rather than replaced by that proof. Historical `dot_double_box_pysecdec_target.json`
contains manually recorded central values with zero placeholder errors and is
not an independently measured pySecDec reference despite its filename.

## Off-shell triple-box: use the existing native propagator API

The exact family proof in `native-family-projection-diagnostics.md` establishes
the following original-order power replacement with coefficient one:

```
original:  [1,1,1,1,1,1,1,1,1,1]
projected: [1,1,0,2,0,2,1,1,1,1]
active:   [1,1,2,2,1,1,1,1]
```

The original inverse-denominator product and the native projected product are
canonically equal, with unchanged loop/external bases and exact Gram matrix.
The independent pySecDec constructor accepts the original ordered propagator
list and the projected ten-entry powerlist: its
`loop_integral/from_propagators.py` explicitly removes zero-power propagators and
their Feynman parameters. Its own Gaussian numerator machinery then produces the
reference density. Do not feed FastSecDec's Gaussian result or sector maps into
the oracle.

Use the already existing `dot_parser.parse_dot_file` and
`kinematics.load_kinematics` helpers to retrieve the original literal routing and
the exact integer/half-integer off-shell replacements from
`examples/graphs/triple_box_offshell_rank2_numerator.dot` and
`triple_box_offshell_kinematics.yaml`. The reference-only native API configuration
is:

```python
li = LoopIntegralFromPropagators(
    parsed.graph_attr_list("propagators", separator=";"),
    loop_momenta=parsed.graph_attr_list("loop_momenta"),
    external_momenta=parsed.graph_attr_list("external_momenta"),
    Lorentz_indices=parsed.graph_attr_list("lorentz_indices"),
    numerator="1" if scalar else parsed.numerator,
    replacement_rules=kin.pysecdec_replacement_rules(),
    Feynman_parameters="x", regulators=["eps"],
    dimensionality="4-2*eps",
    powerlist=[1,1,0,2,0,2,1,1,1,1],
)
loop_package(
    name, li, requested_orders=[0], real_parameters=[],
    processes=1, form_threads=1, contour_deformation=False,
    decomposition_method="iterative", enforce_complex=True,
    pylink_qmc_transforms=["korobov3x3"],
    package_generator=ordinary_package,  # pySecDec.code_writer.make_package
)
```

Here the kinematics helper has already supplied fixed numeric replacements, so no
additional real-parameter values are needed. Record those replacements and the
constructor's surviving propagators/powers as preparation evidence. The rank-two
numerator is exactly
`k1(mu)*k3(mu)+2*k2(nu)*p1(nu)*k2(rho)*p2(rho)`; the scalar uses the same routing
and numerator one. The off-shell point is all four external squared momenta minus
one and `s12=s23=-2`. The original graph weight is one. pySecDec owns the raised
power measure and Gamma factorials; do not attach an extra projected Jacobian or
factorial by hand.

Build the ordinary native `*_pylink.so` with one make job, then use native
`IntegralLibrary(together=True)` with explicit N8311/R32, seed20261201 for scalar
or20261202 for rank two, Korobov3, no fit function, relative tolerance `0.01`,
absolute tolerance `1e-12`, and maximum-evaluation steering `265952`. The
180-second numerical bound is clipped to the remaining 600-second whole-attempt
budget. If provider logs confirm the initial allocation, 265952 counts scalar
summed-coefficient point evaluations per coefficient, not full-vector work or
an accuracy certificate. Keep the complete original string tuple before native
conversion and compare its full physical result. This direct API
glue belongs only in ignored external-reference output. The frozen DOT parser
rejects nonpositive edge powers; editing DOT edges to zero would not be a valid
way to access this native constructor. No new graph parser or denominator
deduplication is needed.

## Positive-orthant inputs: existing all-sector native route

The original Issue 1 preparation is retained here as historical source evidence.
Its bounded initial attempt and separately copied continuations have completed;
see the [attempt record](remaining-reference-attempts.md) and
[ordinary sector-sum outcome](issue-one-together-outcome.md). The launcher hashes the native card
and polynomial alongside the external card/U/F parser and records the explicit
domain and requested highest order. Its frozen source SHA-256 is
`7fd3ba1ce6acc8fedede7c8a875e6f2dd110037ac596bedd4c6e09b96c1cdf61`.
Operational preflight found the old GCC/M4 Nix paths had been collected. The
launcher now uses verified existing GCC 15.3.0 and M4 1.4.21 paths, retains GNU
Make 4.4.1, and records each actual executable path/hash/version. No scientific
package setting or integration parameter changes.

The directory `output/diagnostics/remaining-pysecdec/issue1-attempt-1` now contains
the retained failed initial attempt and must never be reused. Its original command
below records the historical source route. The prescribed 600-second whole-process,
30-GiB process-tree memory and 180-second numerical bounds remain unchanged;
the native disteval request begins at 8192 points and 32 shifts. One allowed CPU,
one package worker and singleton native thread settings preserve serial symbolic
initialization. This is independent scientific evidence, not matched timing.

The input is exactly seven-dimensional positive-orthant `F^(eps-2)` with unit
prefactor and no sector filter. The highest requested order is **+2**; retain
every returned Laurent order and uncertainty before comparison. The existing
native complete vector is `[0,1,2]`, not a padded negative-order vector. Do not
invent missing external coefficients as zeros. The returned native disteval
object is captured before the unchanged parser, then its complete physical
result, all-sector metadata, actual work/unknowns, generated prefactor and
absence of contour deformation require independent outcome review. Historical
manual decimal targets remain unverified and uncertainty-free; they do not
substitute for this reference.

Use the common environment/watchdog from the double-box wrapper, but the following
existing CLI invocations, each with a distinct output directory:

```sh
.venv/bin/python FSD.py run \
  --run examples/runs/issue_1/four_loop_test_int_1.toml \
  --dot-engine pysecdec --workers 1 \
  --normaliz-executable .venv/lib/python3.12/site-packages/pySecDecContrib/bin/normaliz \
  --samples-per-iter 8192 --qmc-shifts 32 \
  --pysecdec-epsrel 0.01 --target-integration-time 180 \
  --pysecdec-workdir "$REFERENCE_ATTEMPT/package" \
  --result-path "$REFERENCE_ATTEMPT/result.json" --json --no-progress

.venv/bin/python FSD.py run \
  --run examples/runs/four_loop_hard_all_sectors_pysecdec_native.toml \
  --workers 1 \
  --normaliz-executable .venv/lib/python3.12/site-packages/pySecDecContrib/bin/normaliz \
  --samples-per-iter 8192 --qmc-shifts 32 \
  --pysecdec-epsrel 0.01 --target-integration-time 180 \
  --pysecdec-workdir "$REFERENCE_ATTEMPT/package" \
  --result-path "$REFERENCE_ATTEMPT/result.json" --json --no-progress
```

Their source cards specify `geometric_infinity_no_primary`; this is the full
positive orthant, not a projective simplex or unit cube. Issue 1 is the complete
density `F^(eps-2)` in seven variables through order two. The hard case is
`U*F^(eps-3)` in nine variables through order zero. Both supplied global prefactors
are exactly one, so the current U/F package builder's omission of a separate
prefactor does not change these two inputs. Do not extrapolate that observation
to arbitrary U/F cards.

The U/F bridge uses native **disteval**, not the DOT `IntegralLibrary` lane.
Here `samples-per-iter` and `qmc-shifts` set initial per-kernel points/shifts;
`target-integration-time` supplies the numerical deadline. `pysecdec-maxeval`
does not bound this lane. Preserve its native per-kernel logs and actual adaptive
work instead of reporting the initial request as the final sample count. The
all-sector flag must be present in the final raw report. The hard `psd2807` card
is a single raw sector and cannot reference the full integral or a native sector
with a coincidentally equal index.

These original launcher commands are retained for traceability; Issue 1 has
executed and its directory must not be reused. Hard four-loop remains unexecuted.
The old triple-box draft below is superseded for new execution by
`run_projected_triple_together.py`. No unexecuted command may launch without the
coordinator's runtime handoff:

```sh
DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder/.venv/bin/python \
  output/probes/run_remaining_reference.py issue1 \
  /common/dev/fastsecdec/output/diagnostics/remaining-pysecdec/issue1-attempt-1

DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder/.venv/bin/python \
  output/probes/run_remaining_reference.py hard4loop \
  /common/dev/fastsecdec/output/diagnostics/remaining-pysecdec/hard4loop-attempt-1

DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder/.venv/bin/python \
  output/probes/run_remaining_reference.py triple-scalar \
  /common/dev/fastsecdec/output/diagnostics/remaining-pysecdec/triple-scalar-attempt-1

DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder/.venv/bin/python \
  output/probes/run_remaining_reference.py triple-rank2 \
  /common/dev/fastsecdec/output/diagnostics/remaining-pysecdec/triple-rank2-attempt-1
```

The common ignored launcher refuses existing output directories, records exact
argv, package/tool/source versions and hashes, fixes affinity/native threads to
one, and delegates the 600-second/30-GiB child-tree bound to the unchanged
reference watchdog. Hard-case preparation includes the inherited YAML card in
its source hashes. Each completion records its exit status and generated
disteval/package/prefactor evidence, whether successful or failed.

For the U/F routes, `capture_pysecdec_disteval.py` observes the argument to the
unchanged `_parse_disteval_coefficients` parser and closes the full native result
files first. That parser fills missing intermediate orders with zero; acceptance
must inspect the retained raw `sums` keys and never mistake such a placeholder
for an independently established coefficient. Native per-kernel logs are retained
for actual adaptive counts and coverage; initial 8192 points and 32 shifts are
not reported as final work.

The original projected triple-box draft, `projected_triple_reference.py`, calls the
existing DOT/kinematics readers and native constructor described above. It
records original propagators, ten-entry projected powers, native surviving
propagators/powers/parameters, exact replacement strings and the native Gamma
factor before generation. Scalar and numerator runs use explicit distinct native
QMC seeds 20261201 and 20261202, respectively. The complete `IntegralLibrary`
tuple is stored without another prefactor convolution or hole-filling parser;
tuple entry two is recorded as the native physical-result entry, subject to the
same generated-prefactor audit. That draft's `maxeval=65536` was steering, not a
measured work count. The current ordinary-constituent runner replaces those
implicit defaults with the explicit allocation above and samples the sector sum
before estimating uncertainty. Neither triple-box runner has executed.

The existing hard builder decomposes both U and F even though U has regular
integer power one; this can cost more than native FastSecDec's justified F-only
fan. If the full attempt exceeds its bound, a subsequent separately reviewed
external configuration can use pySecDec's existing `other_polynomials=[U]`
capability with only `F^(eps-3)` decomposed. This is not permission to replace a
failed attempt silently, and no such alternative has run.

## Acceptance and preservation

Preserve every real/imaginary coefficient, its stated uncertainty, requested and
returned orders, full-integral scope, input/dependency/package hashes, numerical
steering, actual observed work where available, and any nonzero imaginary output.
Unknown reference uncertainty stays unknown; a zero floating-point error estimate
does not establish an exact coefficient. Never fill omitted orders with zero
without an independent structural proof. The historical manual targets remain
unverified.

Convert a successful reference to the existing typed `ReferenceResult` only after
checking its complete physical normalization, source/kinematic identity and
native generation/integration success. Keep validation evidence bounded to the
checks actually performed. Use the library's key alignment and uncertainty
comparison, with explicit independence evidence and retained native covariance;
do not introduce a second estimator or declare a reference exact. For repeated
reference refinement, prespecify independent native seeds and retain every run.
A greater-than-five combined-error discrepancy triggers investigation, not
favourable reruns or omitted rows. The completed double-box reference is recorded
above. The first subsequent Issue 1 execution and transport pass, but its omitted
cross-kernel covariance keeps that fixture Unverified; see
[the independent outcome audit](remaining-reference-independent.md).
The separate `issue_1_together.json` observation has independently checked
input, ordinary sector-sum uncertainty-path and transport evidence; its highest
requested order still misses the one-per-mille target. Neither observation
certifies general error calibration or overwrites the other.
The other full-vector gates remain pending. No timing in this protocol is a
matched FastSecDec performance claim.
