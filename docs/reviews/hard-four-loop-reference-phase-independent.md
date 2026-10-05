# Independent review of the hard-four-loop phase wrapper

Source review accepts the ignored `output/probes/run_hard_reference_phases.py`,
SHA-256 `e7549b4ad525a1b120cab752284f9615b048acf2322bbd4d6249587d8bf45d6b`,
for the separately coordinated bounded attempt. No scientific execution or
reference value is established by this review.

The wrapper invokes the unchanged scientific caller
`run_hard_four_loop_together.py`, SHA-256
`f2d2a98b9f16d02c79c2dbe6400049145a8bebed22b73141c37a4be44293e83b`.
That caller reuses the native request validator, U/F input loader and polynomial
helper. Frozen U/F sources are checked against the Rust inputs. The complete
density is `U*F^(eps-3)` on nine ordered positive-orthant coordinates, with unit
prefactor and all orders through zero. F drives the geometric fan; native
`other_polynomials` handling retains U, including its transformed monomials at
infinity. No projective constraint, graph measure, extra Gamma prefactor or
sector selection is introduced.

The new code changes orchestration only: generation has 600 seconds on CPU0,
guarded native make has 1,200 seconds on eight verified distinct physical cores,
and guarded ordinary integration has 750 seconds on CPU0, inside a 2,600-second
outer watchdog. Every stage is clipped to the remaining total. The negative
Symbolica-import control and ordinary import control precede generation; after
generation, the explicit child-group reap record precedes potentially expensive
package inspection. The shared inspector reads contiguous generation-owned
`sectorN.h` and `sectorN.d` inventories, whose rank-two missing-input controls
already passed. It does not require FORM outputs before their producing stage.

Input, native make arguments and actual generated dimension/order metadata are
validated before compilation, without assuming a sector count or lowest order.
Scientific source and tool hashes are checked between phases and after the outer
process. A fresh output directory, failed/not-started records, package manifests
and raw tuple retention preserve bounded failures without automatic retries.
The guard remains fail-closed; no licensing or dependency workaround is present.

The ordinary constituent uses `together=True`, preserving the whole sector sum
inside each native coefficient estimator. The complete original tuple is saved
before native conversion, and physical member two owns the prefactor application.
Seed 20261219, Korobov3, no fit, published `cbcpt_dn1_100`, requested N8311/R32,
maxeval265952 and one worker are unchanged. The caller's argument `180` is an
admission value for an unused ordinary amplitude wall-clock field; the external
750-second numerical watchdog is the actual bound. Work counts, lowest orders,
uncertainty and covariance availability must be read from the completed native
result. This source acceptance does not certify completion, calibration, the
one-per-mille highest-order target or performance.

## Completed generation preflight

The retained attempt-1 generation inspection reports 2,676 contiguous native
sector headers, nine integration dimensions and requested highest order zero.
Native metadata has 2,676 scalar kernels at order zero, 2,099 at minus one,
628 at minus two and 85 at minus three: 5,488 scalar coefficient kernels in
total. The captured input retains the full positive orthant, F-only fan, U as
the extra factor and unit prefactor. An independent whitespace-only comparison
of captured native U/F text with both frozen FastSecDec polynomial files passes.
This is parameterization and generation evidence, not a numerical result.

The external package's order minus three must remain explicit in any subsequent
physical tuple and reference transport. The retained FastSecDec estimate starts
at minus two. A comparison must retain the full order union and distinguish an
unavailable coefficient from an independently proven zero; metadata alone
cannot justify deleting an external pole or padding a missing native estimate.
The subsequent failed numerical outcome is recorded below.

## Retained terminal outcome and separate numerical continuation

Generation exited zero in 243.998015 seconds and compilation exited zero in
843.013452 seconds. Numerical execution reached its original 750-second bound
and returned timeout 124 after 750.952720 seconds, including shutdown. The outer
process returned the same failure after 1,847.304765 seconds within its declared
2,600-second bound. No complete original tuple or physical result exists.

Native `pylink_integral.hpp` prints `Summing integrands` before `std::accumulate`
over the complete sector Series; its later `Integrating` marker precedes
`deep_apply(all_sectors, integrator->integrate)`. The first QMC timestamp is
635.818 seconds. Three scalar coefficient calls finish at 265,952 points each;
the fourth starts at 731.038 seconds and is interrupted. The setup interval is
not QMC work. Rounded intermediate log values are retained as diagnostics only,
never promoted to a partial reference or used to omit the finite coefficient.

Independent review rehashed all **43,518** package files (**692,292,862 bytes**)
and all **33** recorded scientific/tool sources. The final package exactly
matches the compiled manifest, the four recorded child PIDs are absent, and
the complete tuple/result files are absent. The frozen parent review is
`hard-four-loop-together-attempt-1/independent-review.json`, SHA-256
`5b36eca998d915e90ff6a7d70f1470f9a25e023ddb4b6a7681f318b4ef2b5e43`.
This accepts an honestly retained failed attempt, not a scientific reference.

The separately proposed `retry_hard_four_loop_numeric.py`, SHA-256
`e675a610d931d79f511d42bf69fede88c6dad39bee687af45723bd8466d8fce1`,
has no source blocker. It admits only that completed compiled package and
failed numerical parent, copies every byte and nanosecond mtime into a fresh
directory, and rechecks parent records, sources and library identity. It repeats
the fail-closed import controls and invokes the unchanged guarded ordinary
caller with the original seed and `together=True`. There is no generation,
compilation, pooling of incomplete calls or changed numerical allocation.

The fresh bounds are 1,200 seconds for numerical setup/loading/integration and
1,250 seconds for the watched stage process, clipped to its remaining budget,
with the existing 30-GiB process-tree limit on CPU0. Preparation copying/hashing
precedes that watched process and is not included in its timing. Native
`wall_clock_limit=180` remains an unused ordinary-provider field; the external
watchdogs are authoritative. The old 750-second record stays immutable and the
new run must still return all physical orders minus three through zero before
reference acceptance. Source approval establishes no guarantee of completion.
