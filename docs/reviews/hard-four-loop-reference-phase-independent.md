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

## Completed physical reference and separate native zero certificate

Attempt two returns the complete ordinary physical tuple at index two. Its
numerical process exits zero after 1,101.467362 seconds within 1,200 seconds;
the outer watched process exits zero after 1,105.096355 seconds within 1,250.
The library reports 0.066577 seconds for loading and 1,100.355518 seconds for
integration, including summed-integrand setup. The first QMC call begins at
731.278 seconds. All four native calls report 8,311 points and 32 shifts, or
265,952 scalar summed-sector coefficient evaluations each. The total 1,063,808
uses that unit; it is neither a common full-vector point count nor multiplied
by the 2,676 sector count. Auxiliary setup is excluded from that work count.

Independent outcome verification rehashes all 43,518 package files / 692,292,862
bytes, 35 current source/tool files and 53 immutable parent records. The copied
package still equals its complete initial inventory. The native ordinary
`together=True` path sums sectors before per-coefficient uncertainty estimation;
the recorded library is real-valued, with no separately supplied imaginary or
cross-order covariance. Unit prefactor is applied exactly once: raw tuple zero
and physical tuple two agree, and tuple one is the explicit unit series. No
graph Gamma factor or projective measure is introduced into the supplied
`U*F^(eps-3)` positive-orthant density.

All four original decimal value/error pairs match the converter's hexadecimal
float values bit for bit. The transport audit retains exact `u64` bits and
round-trip decimal numbers, including the explicit `-3` row `0 ± 0`:

| Order | Provider mean | Provider standard error |
| --- | ---: | ---: |
| −3 | 0 | 0 |
| −2 | −3.5988076591592764 | 0.0097315319413335457 |
| −1 | −16.665500979650915 | 0.089032231393958952 |
| 0 | −150.88953098736903 | 0.85859458240279429 |

Finite relative standard error is 0.00569022, so the user's 1‰ target is not
met. Source/normalization/uncertainty-route transport may be Checked while
general statistical calibration remains unverified. The frozen outcome audit
is `hard-four-loop-together-attempt-2/independent-review.json`, SHA-256
`915c1e4604f63351ef0e9507614185f85ad60b1a836540839091ddd683feee95`.

The old native numerical estimate contains only `[-2,-1,0]`. It is not padded,
and the ordinary full-union reference comparison retains MissingEstimate at
`-3`. A separate normal-library proof under
`output/diagnostics/hard-lower-order-zero-attempt-1` regenerates the identical
factorized density with highest order `-3`. It completes all 699 native
subtraction/Laurent representatives and all 2,760 ordered charts, returns
literal zero and no numerical kernels, and passes exact portable-metadata
comparison after removing only kernel association. Successful native generation
enforces its Laurent remainder contract; no test-only skip/capture hook is linked.
Independent postflight hashes pass. This is explicit zero-through-`-3` evidence,
not an inference from the provider's zero estimate. The process exits zero in
67.388249 seconds; the initial failed feature-variant link is preserved and the
successful build binds the correct normal-library dependency fingerprints.

Native transport now passes independently. The encoder output is byte-identical
to `examples/references/four_loop_hard.json` (SHA-256
`328d55c9efc9760a24fb5ee03b9b2bccee5bf2aa99ecc2984b188f6e727ecd73`).
The writer asserts all eight provider IEEE bit patterns before native encoding,
preserves the complete old estimate and all nine covariance entries, and keeps
the `-3` observation as statistical `StandardError(0)`, not `Exact`. Its
full-union comparison is ineligible solely for MissingEstimate at `-3`;
common-order diagnostic pulls are −1.62849, −1.40170 and −1.25618. The separate
zero certificate is provenance evidence, never substituted into either vector.
Imaginary observations and cross-order covariance remain unavailable.

The focused reference target passes seven tests with two ignored recording
probes. Its existing `support::load_reference` verifies every recorded native
card/U/F source hash before comparison, so the new test reuses that validation
rather than duplicating it. Independent writer-build hashes and fixture/output
equality pass; `independent-transport-review.json` records the unchanged layout
and covariance checks. Calibration and the 1‰ finite-part goal remain open.
