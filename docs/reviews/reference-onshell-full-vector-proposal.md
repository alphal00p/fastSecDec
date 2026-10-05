# Original on-shell triple box: reference inventory and minimal reuse

Initial read-only inventory and source proposal, 2026-10-05, after `22dc1d9`.
The subsequent authorized ten-parameter attempt and its retained failure are
recorded below; the projected follow-up remains source-only.

## Existing evidence

No Checked full-integral reference for the original on-shell
`examples/runs/triple_box.toml` was found in the native fixtures, retained
reference attempts or supplied Pathfinder output directory. The two Checked
triple-box fixtures are explicitly **off shell**: scalar and rank two, four
external virtualities −1 and `s12=s23=-2`. They cannot validate the original
on-shell point at `s12=s23=-1`. The accepted original-expression three-point
oracles validate one captured representative, not an integrated full graph.

The unchanged Pathfinder `examples/runs/dot_triple_box.yaml` has no target
entry. Its supplied `examples/outputs/` contains triangle, box and double-box
targets and the hard-polynomial report, but no triple-box result or prepared
bundle. `exhaustive_README.md:347` describes a historical 1,972-sector bundle
and a one-point diagnostic that completed 1,746 sectors and capped 226. The
README itself labels this as subtraction/stability evidence rather than
precision validation. Those reported timings and unavailable bundle do not
constitute a newly reproduced reference or matched generation measurement.

The existing [input-parity audit](scientific-fixture-parity.md) establishes the
normalization to preserve: three loops, ten original massless unit-power
edges, null external legs, `D=4-2*eps`, unit graph/measure multiplier, normalized
per-loop measure `d^Dk/(i*pi^(D/2))`, and `Gamma(4+3*eps)` exactly once. The
original graph includes degree-two vertices `v4` and `v6`; its repeated
propagators are documented in the
[native topology audit](triple-box-offshell-diagnostics.md).

## Literature check

The supplied Pathfinder source/docs contain no analytic full-vector target
for this graph. [Smirnov's 2003 result](https://arxiv.org/abs/hep-ph/0305142)
gives the unit-power planar triple-box ladder through the finite term. The
supplied graph's repeated propagators prevent using that result merely because
the fixture is named `triple_box`. The broader
[Henn–Smirnov–Smirnov master-integral results](https://arxiv.org/abs/1306.2799)
provide planar massless on-shell three-loop families through weight six.
This inventory establishes no exact family mapping/reduction and normalization
to their basis, so those papers are possible independent sources, not accepted
numeric fixtures. No new reduction, differential-equation or polylogarithm
implementation is proposed to obtain this reference.

## Smallest existing external route

Reuse the ordinary constituent path already validated for the scalar/rank-two
and hard references, keeping all external dependencies confined to ignored
development probes. A new narrow caller can reuse
`output/probes/run_projected_triple_together.py`'s ordinary package generation,
`IntegralLibrary(together=True)` numerical call and raw tuple capture. Replace
only its scientific input constructor and case identity:

1. Load the original Pathfinder `triple_box.dot` and
   `triple_box_kinematics.yaml` with its existing `parse_dot_file` and
   `load_kinematics`. Use `ParsedDotGraph::pysecdec_internal_lines()` and
   `pysecdec_external_lines()` with native `LoopIntegralFromGraph`, ten unit
   powers, regulator `eps`, dimension `4-2*eps`, and the existing replacement
   rules. This is the scalar constructor already used by
   `src/pysecdec_bridge.py::_make_loop_integral`; no new parser, graph or
   projection is needed. Retain exact native input/U/F/measure metadata and
   bind the corresponding Rust card/graph/model/parameter bytes.
2. Pass that native integral to `loop_package(requested_orders=[0],
   additional_prefactor=1, package_generator=ordinary_package)` with the
   existing iterative, no-contour, complex-output and Korobov3 settings.
   Native `loop_package.py` supplies `additional_prefactor *
   loop_integral.Gamma_factor`; do not insert another Gamma or scale factor.
   Inspect actual generated order/sector metadata, never assume the off-shell
   four-order range or its 1,182-sector count. Generation owns `.h`/`.d`
   inventories; FORM's `.info` products are checked only after compilation.
3. Reuse the guarded native build and ordinary `IntegralLibrary` call with
   `together=True`, all sectors, independent seed and actual logged work.
   Preserve the unmultiplied tuple, native prefactor and **physical tuple
   member 2** unchanged. The physical tuple already includes Gamma once.
   Retain every returned order and real/imaginary observation, provider errors
   and unavailable cross-order covariance. Do not use the earlier disteval
   variance route or pool successive attempts.
4. Reuse the reviewed scalar reference writer's native `ReferenceResult`,
   encoder/reader and `compare` APIs. Bind audited provider values/IEEE bits,
   physical normalization, complete package/allocation and source hashes.
   Compare the full signed-order union with the completed native original
   integral; missing rows remain explicit unless separately proved absent.
   Checked transport, estimator calibration and one-per-mille convergence
   remain distinct claims.

The current phase wrappers, corrected generated-header inspector,
`guard_reference_python.py`, process-group watchdog and immutable-copy
continuation machinery can supply scheduling and provenance unchanged in
meaning. Generation takes the exclusive symbolic slot; later guarded FORM/C++
and numerical work must not import Symbolica. Concrete phase deadlines,
toolchain paths and immutable inputs belong in a fresh preflight before any
execution; this source proposal authorizes none and extends no old deadline.

The narrow next reference step is this one original-input ordinary observation;
it need not wait for successful native full-graph generation. Only their
complete-vector comparison requires both results. Under coordinated CPU and
symbolic-runtime ownership, guarded external compilation/numerics may overlap
native capability work, with the overlap recorded and no timing comparison.
The existing
[Pathfinder generation-only protocol](reference-onshell-generation-source.md)
serves the separate generation-baseline question; it is not a substitute for
an integral reference. Neither step requires changing the physical card or
production dependency graph.

## Concrete ignored source draft

The initial source implementation was placed under
`output/probes/onshell_full_reference/`: `common.py` owns the original input
digests/settings and imports existing orchestration helpers, `science.py` owns
the thin native constructor/ordinary numerical call, and `run.py` owns fresh
preparation, native metadata inspection, bounded phases and provenance.
AST syntax checks passed before any module import control, build or scientific
execution. Pure-data preparation is recorded below. Existing frozen callers and
packages were not modified or copied. Only the three small new source files were archived during
preparation; generated package manifests are recorded at stage boundaries.

Proposed limits, set before any launch, are 600 seconds generation, 1,200
seconds native compilation and 1,200 seconds ordinary numerical evaluation,
under one 3,100-second outer bound. The existing watchdog enforces 30 GiB
simultaneous process-tree RSS with five-second interrupt and kill grace
periods; this differs from the native campaign's address-space bound.
Generation/numerical work use CPU0; native make uses CPUs0–7 with eight jobs and
one FORM thread each. These are independent capability bounds, not a matched
benchmark. The numerical allowance reflects the retained hard-reference
setup/summation experience; it does not claim this input needs that duration.

The frozen numerical request uses the existing Korobov3/`cbcpt_dn1_100`
configuration, `minn=8311`, `minm=32`, `maxeval=265952`, independent seed
`20261205`, one numerical worker, `epsrel=0.01` and `epsabs=1e-12`.
Actual logged work and all returned physical orders remain authoritative;
these requests are neither a fixed complete-vector count nor a one-per-mille
acceptance claim. External watchdogs remain authoritative because ordinary
`pylink_integral.hpp` does not enforce the amplitude wall-clock option.

Preparation and execution are separate commands:

```sh
DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder/.venv/bin/python \
  output/probes/onshell_full_reference/run.py --prepare NEW_ABSOLUTE_ATTEMPT
DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder/.venv/bin/python \
  NEW_ABSOLUTE_ATTEMPT/sources/run.py --run NEW_ABSOLUTE_ATTEMPT
```

The proposed `--prepare` is data-only and checks the exact original input
digests plus the accepted existing compiler/tool paths. The subsequent run
checks its frozen manifest, rejects prior stage/package outputs, serializes
guard/import controls and the three scientific stages, records explicit
generation exit/reap before inspection, and stops on the first failure.
Corrected `.h`/`.d` generation admission is reused. At that stage, execution
remained pending concrete preflight and the coordinated runtime handoff.

Independent HEPKit source review accepted this narrow preparation slice. It
checked original input/Gram/powers, native Gamma ownership, generated header
inventory, ordinary complete-sector summation before QMC (`pylink_integral.hpp`
lines 163–180), tuple transport and external deadlines. `loop_package.py` line
96 applies the additional prefactor times the native Gamma factor. This is
source acceptance, not numerical validation or uncertainty calibration.

Pure-data `--prepare` then exited zero without importing Symbolica or starting
a child science process. The fresh campaign is
`output/diagnostics/remaining-pysecdec/triple-onshell-together-attempt-1/`.
Its 45-entry `frozen.sha256` passes author verification and has SHA-256
`c4c6a857660422931fc98a3f4162d3bdb7df87478d5d8f337f4b2ffd44517fb7`;
`preparation.json` is
`c274a0b3eaefed4b5720fbcfb972e84fbaf26c2247398febca3c13dd9cfa56e0`.
The three executed-source candidates are archived under `sources/`; external
helper/dependency/tool and seven physical input identities are hash-bound.
Production source and documentation are not watched mutable inputs, so a later
documentation commit does not relabel or invalidate the recorded `22dc1d9`
preparation. At preparation completion, the package directory was empty and
all scientific process, generation and result records were absent.

## Original ten-parameter attempt: memory-bound failure

The explicitly authorized attempt subsequently ran once and stopped at its
declared 30-GiB process-tree RSS cap, before the 600-second generation deadline.
The generation watchdog sampled 30.075 GiB at 517.4 seconds; the outer watchdog
sampled 30.109 GiB at 520.0 seconds and interrupted the same tree. The outer
process returned 137 after 530.287582061 seconds. All four recorded process
groups, including generation PID/group 2724214 and outer group 2724174, were
confirmed absent in `author-reap-verification.json`. No process survives.

The native constructor record preserves ten unit-power edges, the original
on-shell Gram replacements and `Gamma(4+3*eps)`. Native output reports 968
sectors after symmetry and reaches FORM input writing for sector 6. This is a
logged stage count, not a completed-package certificate. The
interrupted traceback is inside pySecDec's IBP derivative/ProductRule copying;
this locates the interruption, not a complete cost profile. Seventeen partial
package files totaling 18,312,400 bytes remain, with no generation-complete
marker, accepted package
inspection, library, raw numerical tuple or result. The outer memory stop
also interrupted the inner watchdog/stage writer, so its normal generation
process/reap and later-stage skip records were not produced. They are not
reconstructed as successful records. The outer completion and independent
process-group absence check retain the actual terminal boundary.

All 45 immutable postchecks pass. Native release compilation on CPUs10–11
overlapped only the final portion, beginning 09:31:03.422082 UTC; reference
generation remained on CPU0. No comparative timing claim follows. The symbolic
slot was released after reaping, and no compile/numerical follow-on or automatic
retry started. This attempt yields no full-vector reference or uncertainty
validation. Independent outcome review accepted the retained failure, checked
all 45 immutable files and all four absent process groups, and confirmed the
missing inner terminal records. Its `independent-review.json` has SHA-256
`aa57e4071a6c1c854ca8bb871f90bf4ee3a4a9a5845ab17c5378828c254898eb`;
original evidence remains under the same attempt directory.

## Same-integral follow-up using the existing native projection

The existing native family preparation supplies the next narrow representation.
It removes two redundant Schwinger parameters while preserving the complete
physical integral. The original propagators have exact repeated pairs at
zero-based positions `(2,3)` and `(4,5)`.

The reviewed [native projection](native-family-projection-independent.md) and
[prepared-family API](native-family-preparation-independent.md) delegate to
`IntegralFamily::partial_fraction` and `sector`. Current
`parametric/preparation.rs::prepare_family` admits only one coefficient-one term
with positive retained powers, unchanged momentum bases and native canonical
equality of the complete inverse-denominator product. It changes neither
numerator nor kinematics and introduces no measure factor.

The retained **on-shell** invocation is
`output/diagnostics/triple-box-projected/onshell-scalar.generate.argv.txt`:
`projected-triple-fe3b72a generate scalar onshell ...`. Its frozen source performs
the exact inverse-product and unchanged-basis assertions before calling native
generation. Subsequent Domain/Geometry/Mapping/Laurent status records show those
assertions passed in that invocation. Its later 310.289317-second Laurent timeout
left the final JSON empty; this is not a completed-generation or numerical
record. `build-provenance.json` binds `projected_triple_box.rs`
(`7bd036bc996e35ce27536ea45d63d72ba0e8193de08e55fe1d225e0824bb9f2f`),
the exact on-shell Gram loader `triple_box_input.rs`
(`f725c301a26585e012796d0587c783e4a15af2a4dcecb159e38ac04300d9b138`),
and executable
`380396e5c328d344fc3dd9f50d4edb50cf03f3823c76a5c5368e5d0204dc41a0`.

The accepted ordinary scalar reference constructor in
`output/probes/run_projected_triple_together.py` already uses this projection.
Reuse that constructor with the original on-shell point; its off-shell values
are not a reference for the new attempt.

| Item | Required value |
| --- | --- |
| Original-order projected powers | `[1,1,0,2,0,2,1,1,1,1]` |
| Active original indices | `[0,1,3,5,6,7,8,9]` |
| Active positive powers | `[1,1,2,2,1,1,1,1]` |
| Loop/external basis | `k1,k2,k3`; `p1,p2,p3,p4`, unchanged |
| Scalar numerator and projection coefficient | `1` and `1` |
| Point | `p_i^2=0`, `s12=s23=-1`, original ten exact replacement rules |
| Constructor | `LoopIntegralFromPropagators`, `D=4-2*eps` |
| Prefactor | Native `Gamma(4+3*eps)` once; additional prefactor `1` |
| Requested maximum | epsilon power `0`, complete returned order union |

Read the already audited routing strings from historical
`triple_box_offshell_rank2_numerator.dot`, explicitly replacing its numerator
with `1` and using **original** `triple_box_kinematics.yaml`. Compare its parsed
ordered internal/external line records to `triple_box.dot`: the connectivity,
massless lines and external attachments must match. Require the literal repeated
pairs before passing the bound native power vector, then check the constructor's
eight active parameters/powers. Freeze both graph inputs and the existing native
projection/routing evidence. This introduces no parser, routing or reduction.

Native pySecDec must retain the raised-power measure, including the two parameter
factors from powers two and its `1/Gamma(nu_i)` normalization. Total power remains
ten, so native Gamma remains `Gamma(4+3*eps)`. Ordinary `loop_package`, full-sector
`IntegralLibrary(together=True)` and physical tuple member 2 retain normalization
and error transport. The projective dimension becomes seven; generated sector
and order counts must come from the new complete package, not the off-shell
package or the incomplete 968-sector log.

The authorized next work is source adaptation and pure-data preparation in a
**new** ignored harness/directory. Preserve all failed files. Reuse the existing
separate-stage runner, changing only the constructor and its metadata admission;
start an empty package. Keep 600 seconds generation, 1,200 seconds each for
compile/numerical work, 3,100 seconds outer and 30 GiB process-tree RSS, with the
same guards, CPU allocation, seed and QMC request. No observations are pooled.
Execution remains unscheduled until independent concrete review and root's
symbolic-runtime handoff; native fullgraph generation has priority.

Independent HEPKit review confirmed reuse of the already executed on-shell
identity guards, including their source/build/argv/status binding. A new draft
now exists under `output/probes/onshell_projected_reference/`; AST parsing passes
without importing the scientific dependencies. Its `common.py`, `science.py`
and `run.py` SHA-256 prefixes are `e289a131`, `5cdf74e2` and `88a59374`.
The source adds only the concrete projected constructor/measure checks and
seven-dimensional package admission to the existing phased harness. The same
full-sector ordinary numerical call remains intact. Independent HEPKit source
review accepted the concrete delta, including native `common.py:290–326`
ownership of the two raised-power measure factors and Gamma normalization.

Pure-data preparation then exited zero for
`output/diagnostics/remaining-pysecdec/triple-onshell-projected-together-attempt-1/`.
All 57 frozen checks pass, covering 56 source/tool/input entries plus preparation;
`frozen.sha256` has SHA-256
`abe11bf2149c04357e72e7bc9d5d0f4c7254320eb04a2aa6148ecffdff6efeae`,
and `preparation.json` is
`47bbfedc52c5907f05bb4457faef449b993e182b6311337e2f28faf04710b6e1`.
The three archived sources match the reviewed hashes. The package is empty and
all scientific records remain absent. Only the old small proof/source records
and fixed executable are read; no generated package is copied. Preparation
records native revision `a56107f` and provider `582d8c7`, with exact source/tool
hashes carrying execution identity independently of later documentation commits.

The eventual invocation is the archived `sources/run.py --run` with that
absolute attempt directory, using the existing reference venv Python.
Independent concrete preflight accepted all 57 immutable checks, the native
identity-evidence binding, unchanged settings/bounds and the fresh package;
`independent-preflight.json` has SHA-256
`da2d968ac0e459c0797fb73e8667416e657f4d7c4b12250b29809ef1e3a91259`.
At that preflight no scientific process had run. The root subsequently
authorized the single invocation below after native fullgraph released the
exclusive symbolic runtime; preflight itself accepted no numerical value.

## Projected attempt: memory-bound generation failure

The prepared projected attempt ran once and also stopped at the declared
30-GiB aggregate process-tree RSS cap, before its 600-second generation limit.
The outer watchdog sampled 30.085 GiB at 421.5 seconds; its process returned
137 after 431.800959838 seconds. `terminal-slot-release.json` records the
reaped wrapper and all four absent process groups (3328794, 3328797, 3328989,
3329016). All 57 immutable postchecks pass. No process survives and no
compile/numerical follow-on ran.

The native input record retains the exact eight-parameter projection, raised
powers/measure and `Gamma(4+3*eps)`. Native output logs 968 sectors after symmetry
and FORM input writing for sector 2; this is not a completed-package
certificate. Nine partial files totaling 1,263,214 bytes remain. The interrupted
trace again lies in recursive native IBP/ProductRule copying. There is no
generation-complete marker, accepted package inspection, library, raw tuple or
numerical result. As with the first attempt, the outer memory stop interrupted
the inner stage writer, so normal generation process/reap and later-stage skip
records are absent; the outer completion and group-absence record retain the
actual terminal boundary. The failure supplies no full-vector reference.
Independent review verified all 57 postchecks, the four absent groups and
the retained partial/missing outputs; its `independent-review.json` SHA-256 is
`5dd7c8dffdf160856fba59d3791ff47ca03ba0a24267c9b91e50df82b8653cbb`.

## Narrow native pure-Taylor follow-up

The installed ordinary `loop_package` exposes the documented
`ibp_power_goal=-numpy.inf` option. It forwards this value through native
`LoopPackage` into `make_package`; native subtraction then returns each IBP
input before derivative/boundary recursion (while retaining its regulator
validation), and the existing `integrate_pole_part` performs Taylor
subtraction. Independent HEPKit source review confirms this forwarding and
semantics. It avoids the specific recursive IBP branch observed in both memory
failures; it does not establish that Taylor generation will finish or use less
memory overall.

The new ignored draft is
`output/probes/onshell_projected_taylor_reference/`. Its only scientific
steering change is the explicit native option in `loop_package`; the package
name is distinct. The exact projected physical input, original graph/point,
native measure/Gamma, complete coefficient coverage, ordinary full-sector
`together=True` call and tuple member 2 are unchanged. QMC settings and the
600/1,200/1,200-second stages, 3,100-second outer limit, 30-GiB process-tree RSS
cap and existing process guards remain unchanged. This is a fresh package,
not a continuation of either partial package.

The source stores nonfinite steering as the explicit string `"-numpy.inf"`
and a native-route label in both preparation and native-input metadata;
generation inspection requires exact agreement. JSON does not contain a bare
Infinity value. NumPy is already a pySecDec dependency; its installed version
is recorded without importing it during preparation. Five prior frozen/terminal
and independent-audit records are bound as provenance only; no observations or
package files are reused. Source SHA-256 prefixes are `06b71a1f` (common), `67f2adfa` (science),
and `af356772` (runner). AST parsing passes without scientific imports.
Concrete source review and pure-data preparation precede any separately
scheduled invocation. This source note introduced no automatic retry or
deadline extension.

Independent review accepted the three-file source delta. The subsequently
added prior-audit binding changes provenance only. Pure-data preparation
exited zero for
`output/diagnostics/remaining-pysecdec/triple-onshell-projected-taylor-attempt-1/`:
all 62 immutable checks pass, covering 61 source/tool/input entries and
preparation. `frozen.sha256` is
`11b2d51072bfaa8e1988b867f223d2113b3e0f317cf9adee94fea8a09a7ca210`;
`preparation.json` is
`27db992f132988eb71bbf3a40134c793d8367c75bd34a9ab2c0aa5e603903e8f`.
The package is empty, all scientific records are absent, and installed NumPy
2.5.0 is recorded. The immutable source/input identity is independent of later
documentation commits. Invocation remains the existing reference-venv Python
with the new attempt's archived `sources/run.py --run ABSOLUTE_ATTEMPT`,
subject to the concrete preflight and coordinated runtime handoff. Independent
preflight accepted this exact attempt (`independent-preflight.json` SHA-256
`7bcf2ab11c361a360b958b21f3554cd963e8fb5d2ad97aac338ffd8ea6f23366`),
and the root then authorized one run.

## Pure-Taylor attempt: bounded generation timeout

The unchanged 600-second generation deadline fired and the native process
exited after SIGINT. Generation returned 124 after 605.098159458 seconds;
the outer process returned 124 after 608.397574896 seconds. The generation
reap record confirms group 3601732 absent, and all four recorded groups are
absent. All 62 immutable postchecks pass. The symbolic slot was explicitly
released before the prepared native fullgraph trial began.

Native output reached FORM input writing for sector 41 of the logged 968;
this does not certify 41 completed sectors. The interrupted stack is inside
native Taylor/ProductRule derivative simplification. There are 87 partial
files totaling 10,229,949,421 bytes, with no generation-complete marker,
accepted package, library, raw tuple or result. Unlike the prior outer RSS
interrupts, normal generation process/reap and compile/numerical not-started
records were written. Sampled peak RSS was 7.524 GiB for generation and
8.208 GiB for the outer process tree; the last generation sample was 7.170 GiB.
This was a deadline failure, not a memory-cap failure. No deadline was extended
and no subsequent science stage ran.

Independent outcome review accepted the retained failure, all source checks,
group absence and missing outputs. Its `independent-review.json` SHA-256 is
`38dccf2454e5f735d5a3cdc6058f7a94097a73a733ecf6b050e0c7e397e82a48`.
No full-vector reference or uncertainty evidence follows from the partial files.

The next selected source path is the existing Pathfinder direct-evaluator CLI,
whose original on-shell command and immutable formula-cache inventory are
already recorded in
[reference-onshell-generation-source.md](reference-onshell-generation-source.md).
It avoids FORM generation and preserves the original graph and physical
prefactor convention. The source-reviewed native `decomposition_method="geometric"`
alternative would use one Cheng–Wu primary chart but still enters eager
Taylor/FORM generation; it remains a fallback, with no new package prepared.
The planned direct path uses the existing reference environment and one worker,
600 seconds generation and 1,200 seconds numerical work at 30 GiB tree RSS.
It is a correctness reference, not matched timing. Its concrete bundle/input/
full-vector transport remains to be frozen and reviewed before execution.
