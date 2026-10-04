# Six massive multiloop fixtures

This is the preparation record for a bounded production-CLI diagnostic and its
subsequent independent numerical checks. No independently certified values are
claimed here. Raw status streams, artifacts, checkpoints and coefficient vectors
belong under ignored `output/diagnostics/massive-multiloop/`.

## Scientific inputs

All six shipped cards use the native scalar model, the native massive parameter
card (`UFO::mt = 1`), unit propagator powers, dimension `4-2*eps`, explicit measure
multiplier `1`, and requested highest Laurent order zero. They use normalized
per-loop measure `d^Dk/(i*pi^(D/2))`; the native scalar parameterization supplies
`(-1)^N Gamma(N-LD/2)`. All propagator masses are one and all external virtualities
are minus one. Three-point cards set `P0.P1 = 1/2`, with the third momentum fixed
by native momentum conservation. They request ordinary domain certification,
without a no-threshold assertion.

| Native card | Loops | Propagators | External invariants | Expected sign |
| --- | ---: | ---: | --- | --- |
| `kite_2loop.toml` | 2 | 5 | `p^2=-1` | Negative |
| `self_energy_3loop.toml` | 3 | 7 | `p^2=-1` | Negative |
| `three_point_2loop.toml` | 2 | 5 | `p1^2=p2^2=p3^2=-1` | Negative |
| `three_point_2loop_6line.toml` | 2 | 6 | Same | Positive |
| `three_point_3loop.toml` | 3 | 7 | Same | Negative |
| `three_point_3loop_8line.toml` | 3 | 8 | Same | Positive |

The expected finite support is `[eps^0]`. The historical generation tests found
no singular axes for these inputs; their padded negative-order output arrays are
not evidence of actual poles. Native output need not preserve that padding or the
historical sector counts. Exact graph and convention parity is also recorded in
`scientific-fixture-parity.md`.

## Existing reference evidence and reuse

At Pathfinder revision `582d8c7f6dde9bf750750d4c2a2d85a94ce940cd`,
`tests/test_integrals.py::test_dot_multiloop_two_and_three_point_examples_generate_finite_sector_sets`
covers generation of all six cases. The optional
`test_optional_multiloop_fsd_low_stat_compare_to_pysecdec` covers only the kite and
self-energy. It generates a separate pySecDec package at 512 maximum evaluations
and relative tolerance `0.5`, and uses a loose eight-times-summed-error comparison.
There is no stored precision target in that test. It is useful independent
low-statistics evidence once actually executed, not a convergence certificate.

The existing reference CLI supports `--dot-engine pysecdec` and already owns the
package-generation and integration bridge. Its invocation can be confined to the
ignored reference environment; no Python implementation or runtime dependency is
added to FastSecDec. Preserve the existing graph/kinematics files and explicitly
select massive mode, mass one and the pySecDec normalization. Since these inputs
are finite, epsilon-dependent normalization conversions do not mix any pole into
the finite coefficient, but the explicit measure and sign must still agree.

The native HEPKit OneLOop provider supplies A0/B0/dB0/C0/D0. The native
`one-loop-reduce` shared-family admission explicitly requires exactly one loop.
Neither provides these multiloop values. Native Vakint's MATAD/FMFT routes are
vacuum-specific; its general unrecognized topology constructor selects
`EvaluationOrder::pysecdec_only(None)`. These off-shell nonvacuum integrals
therefore have no newly discovered pure-Rust analytic master reference through
those interfaces. Reimplementing a reduction or a numerical integrator is not
justified by this limitation.

## Bounded native diagnostic

The ignored `fastsecdec-cli` test `massive_multiloop` invokes the production CLI
serially for generation, then integrates all successfully generated artifacts.
Each stage has a 180-second watchdog. The numerical allocation is 1024 lattice
points times eight independent shifts per kernel, seed `20261004`, two workers,
Korobov-3 periodization and one refinement round. It saves the complete production
vectors and covariance, numerical diagnostics, structured progress, generation
timings, dependency provenance and resumable checkpoints. Every case is attempted
and a summary is rewritten after each stage, including failures.

The checks require finite real output, the normalized Euclidean sign, complete
production coverage, consistent covariance dimensions and no evaluation failures.
They do not compare sector counts, assume sector numbering, or turn an incomplete
estimate into zero. Passing these checks establishes executable pipeline coverage
only. Independent pySecDec comparisons and increasing-work/independent-seed
convergence checks remain separate gates.

Run explicitly after the current dependency/runtime validation gate:

```sh
cargo test -p fastsecdec-cli --locked --test massive_multiloop -- \
  --include-ignored --test-threads=1 --nocapture
```

Set `FASTSECDEC_MULTILOOP_OUTPUT` to save a distinct diagnostic campaign. Preserve
the artifact and checkpoint identities when performing follow-up integration.

## Executed native diagnostic, 2026-10-04

The ignored harness passed all six generation and integration cases in 26.18 s
using the already-built latest-backend workspace test binary. The CLI used
SymJIT 2.26.4, with exact dependency/source fingerprints retained in each artifact.
All reports contain only the real `eps^0` component, complete production coverage,
and zero evaluation failures. Native whole-vector weighted checks used at most
256-bit MPFR precision. These are development-build diagnostic timings, not
matched performance acceptance results.

| Case | Native kernels | Generation including O2 compilation (s) | Finite coefficient | Estimated standard error | Complete evaluations |
| --- | ---: | ---: | ---: | ---: | ---: |
| Kite, two loops | 4 | 0.114 | -0.680880423296 | 0.000007462366 | 32,768 |
| Self-energy, three loops | 60 | 3.128 | -1.544610335231 | 0.004329327335 | 491,520 |
| Three-point, two loops | 6 | 0.128 | -0.704841717844 | 0.000008670024 | 49,152 |
| Three-point, two loops/six lines | 6 | 0.294 | 0.140936704632 | 0.000900817262 | 49,152 |
| Three-point, three loops | 60 | 3.146 | -1.541524658579 | 0.004352183898 | 491,520 |
| Three-point, three loops/eight lines | 117 | 12.794 | 0.234953909425 | 0.002689172439 | 958,464 |

The two three-loop seven-line cases have 60 native representative kernels, whereas
the historical implementation reported 74 sectors. No equality assertion on those
counts was made. Full source graphs, kinematics, measure and complete integral
values are the comparison boundary.

The ignored log is `output/massive-multiloop-tests.log`; the per-case raw reports,
status streams, portable artifacts and checkpoints are in
`output/diagnostics/massive-multiloop/`. Its summary explicitly records
`independently_certified: false`. External reference and tighter convergence gates
are pending; the numbers in this table must not be treated as frozen reference
truth merely because all native stages completed.

## Initial independent numerical checks

The external reference uses the existing Pathfinder `run --dot-engine pysecdec`
entry point, which returns before constructing FSD evaluators. Its independently
generated C++ package uses pySecDec 1.6.6 and GCC 15.3.0; the frozen reference
environment contains Python 3.12.14 and Symbolica 2.1.0. These are reference-only
versions, distinct from the native product's Symbolica 3.0.1/SymJIT 2.26.4 stack.
The exact command is retained in each raw JSON report under ignored
`output/diagnostics/multiloop-pysecdec/`.

The commands explicitly select massive mode, mass one, iterative decomposition,
no contour deformation, highest order zero, and the normalized pySecDec measure.
The original graph and kinematics files are used at their pinned reference
revision. All successful outputs below have precisely zero imaginary value and
imaginary error; a future real-only frozen transport fixture must retain that
observation in provenance rather than silently discard it.

| Case | External finite coefficient | External reported error | Difference from initial native run / combined error |
| --- | ---: | ---: | ---: |
| Kite, two loops | -0.6808761310038735 | 0.00000005221897285 | 0.575 |
| Self-energy, three loops | -1.5417626421519355 | 0.00001053743661 | 0.658 |
| Three-point, two loops | -0.7048468145599492 | 0.00000005027337491 | 0.588 |
| Three-point, two loops/six lines | 0.1432374698185829 | 0.000002725451247 | 2.554 |
| Three-point, three loops | -1.5387246532088248 | 0.00001065002830 | 0.643 |
| Three-point, three loops/eight lines | 0.2341101403880268 | 0.000105203240819 | 0.314 |

Each initial reference requested `maxeval=4096` and `epsrel=0.05`. These are
steering limits, **not observed evaluation counts**. The existing bridge does not
emit actual lattice sizes or sample counts. Its installed QMC wrapper defaults
to `minn=10000`, with the underlying QMC minimum of 32 shifts when not overridden;
initial work may exceed the requested maximum. The QMC seed is left at its native
zero sentinel, which preserves `std::random_device` initialization according to
the installed `qmc.hpp` and generated pylink template. It is independent of the
explicitly seeded native campaigns. Raw error estimates are retained as numerical
uncertainties, never exact values.

The first self-energy attempt with two CPU cores aborted during package generation
because a second Symbolica instance could not start. The package generator derives
its implicit multiprocessing count from CPU affinity; `--workers` by itself only
controls the later make command in this reference path. A separate retry with
one-core affinity and one make worker completed successfully. The failed logs
remain preserved. No licensing mechanism, unrelated process or dependency source
was changed. Subsequent references use serial package generation.

A higher-work six-line native campaign used 8192 points, 32 shifts, independent
seed `20261005`, two workers and the same Korobov-3 transform. All 1,572,864
evaluations completed without failure, producing `0.1427027551492941` with estimated
standard error `0.0005388525607783324`, about 0.992 combined errors from the
external value. The earlier observation remains recorded. This modest uncertainty
reduction requires investigation of the sector/shift contributions before any
convergence-rate or performance conclusion; it is not evidence of a numerical bug
by itself. The tighter report/checkpoint use the `.tight.*` suffix in the original
native output directory.

The subsequent independent numerical review is recorded in
[`six-line-qmc-convergence.md`](six-line-qmc-convergence.md). Its constant-integrand
controls separate lattice/periodization quality from Feynman kernels and JIT
evaluation. The original observations above remain preserved; the controls do
not replace matched complete-vector convergence checks on this integral.

All six initial independent external comparisons are now complete. The last,
`three_point_3loop_8line`, completed serial generation in 10.722 s, compilation in
213.660 s and numerical evaluation in 3.439 s, within its 600-second watchdog.
These are initial independent correctness comparisons. Tighter reference repeats
and broader convergence evidence remain pending for the campaign; the raw
uncertainties are not convergence certificates.

The serial reference invocation uses the following existing entry point from the
pinned Pathfinder checkout; each raw JSON stores the fully expanded arguments.
`NAME` selects the corresponding historical graph and kinematics file. The kite
used two make workers/two-core affinity; the successful self-energy retry used
the `.serial.raw.json` result and `self-energy-serial-package` directory.

```sh
SYMBOLICA_HIDE_BANNER=1 timeout --signal=TERM --kill-after=15s 600s \
  taskset -c 0 .venv/bin/python FSD.py run \
  --dot-file examples/graphs/NAME.dot \
  --kinematics examples/graphs/NAME_kinematics.yaml \
  --dot-engine pysecdec --mode massive --m 1 \
  --prefactor-convention pysecdec --sector-method iterative --workers 1 \
  --max-eps-order 0 --pysecdec-maxeval 4096 --pysecdec-epsrel 0.05 \
  --pysecdec-workdir /common/dev/fastsecdec/output/diagnostics/multiloop-pysecdec/NAME-package \
  --result-path /common/dev/fastsecdec/output/diagnostics/multiloop-pysecdec/NAME.raw.json \
  --json --no-progress
```

The banner setting changes presentation only. GCC 15.3.0, GNU Make 4.4.1 and M4
1.4.21 were supplied through the local Nix toolchain path. The raw-report SHA-256
identities below anchor the small retained evidence without tracking generated
packages or raw build output:

| Raw report basename | SHA-256 |
| --- | --- |
| `kite_2loop.raw.json` | `2890cb5c984f508719433e6141eaaf319c677f67c40e925fa5335506d8232a99` |
| `self_energy_3loop.serial.raw.json` | `c24729f061983ee27f1dc5126bec8269ba9db60051821a957c351839c71a6c39` |
| `three_point_2loop.raw.json` | `89cc02055c087adfbbddfcc6c4a9733a8e06feeb27962a9fd3853e42999acbe2` |
| `three_point_2loop_6line.raw.json` | `b57bf8c86a85dbe3f51ed4c7e78de0e2baaea087b98d7711317fd9b0960d8490` |
| `three_point_3loop.raw.json` | `516364f26f31b21eede883b6bb59bc0256d0fb2288fc5d214a54f278a31d3b79` |
| `three_point_3loop_8line.raw.json` | `d85baa52bf0ea614212a98269f3ddd3e58952767cb6e352cd13a3f301e9da680` |

## Frozen native transport and comparison gate

The six small versioned transports in `examples/references/` were constructed
through the native `ReferenceResult` and `encode_reference` APIs after the
independent review in `massive-multiloop-reference-independent.md`. Their positive
reported errors remain `StandardError`; the explicitly checked real projection
retains the source imaginary value and error in provenance. `Checked` records
source/input/normalization and bounded comparison evidence, not convergence.
Raw commands, revisions, versions, external/native source digests, and raw-report
digests remain available without tracking generated packages.

Three pure-data tests passed in `output/multiloop-reference-tests.log`: the
transport/input evidence checks, an explicitly ignored candidate recorder, and
all six saved complete-vector comparisons delegated to the native `compare`
API. The recorder writes only ignored candidates and rejects raw-report changes
against the independently audited digest. Normal tests need no external tool or
ignored report. The ignored six-case production-CLI harness now includes these
same typed comparisons with a five-combined-standard-error initial correctness
threshold; its unchanged sampling pipeline remains separate from convergence
acceptance.

The provenance audit corrected the external rule description: installed
`secdecutil::integrators::Qmc` merges dimension-appropriate CBC/PT tables
(`cbcpt_dn2_6`, `cbcpt_cfftw1_6`, `cbcpt_cfftw2_10`, `cbcpt_dn1_100`). It does not
necessarily select only `dn1_100`. The frozen metadata describes that native
default while leaving the actual selected rule and point count unknown.
