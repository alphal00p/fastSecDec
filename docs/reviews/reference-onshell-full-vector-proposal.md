# Original on-shell triple box: reference inventory and minimal reuse

Read-only inventory and source proposal, 2026-10-05, after `22dc1d9`. No
reference code, package, evaluator or scientific process was built or run.

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

The implementation is now source-only under
`output/probes/onshell_full_reference/`: `common.py` owns the original input
digests/settings and imports existing orchestration helpers, `science.py` owns
the thin native constructor/ordinary numerical call, and `run.py` owns fresh
preparation, native metadata inspection, bounded phases and provenance.
AST syntax checks pass; no module import control, build or scientific execution
has run. Pure-data preparation is recorded below. Existing frozen callers/packages are not
modified or copied. Only the three small new source files are archived during
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
Corrected `.h`/`.d` generation admission is reused. The scientific command
remains unexecuted pending concrete preflight and the coordinated runtime
handoff.

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
preparation. The package directory remains empty and all scientific process,
generation and result records remain absent.
