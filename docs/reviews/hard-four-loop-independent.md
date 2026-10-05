# Independent hard four-loop output audit

This reviewer did not run or author the bounded generation/integration campaign.
The review reads `output/diagnostics/four-loop-hard` and invokes the preserved
production CLI's numerical-only `show-result` reader. It initializes no Symbolica
evaluator and repeats no integration. Reader output is retained as
`independent-show-result.json` and `.stderr` in that directory.

## Input, identity and domain

The SHA-256 records for the run card, both polynomial files and preserved
`561657b` executable match the files actually present. The card is the explicit
nine-dimensional positive-orthant density `U * F^(eps-3)`, with prefactor one and
maximum Laurent order zero. It is not a projective graph integral and carries
no implicit loop-measure or Gamma multiplier. The native retained domain is
`PositiveOrthant`/`NoThresholdReal`, with a positive-coefficient certificate for
the noninteger-power F factor and no caller assertion. Negative U appears only
to integer power one; no complex branch was silently asserted.

The artifact, integration report, outer checkpoint and saved-result provenance
agree on outer identity
`6861bd7ada651f6900e00b9c95380db679e38ce7026ef359c22c3ef9f469429e`.
The saved manifest's inner kernel identity matches the artifact. Retained map
metadata contains 2,760 charts and 699 representatives; the numerical manifest
contains all 699 nine-dimensional kernels. Different historical sector counts
are not an equivalence requirement.

## Accepted numerical evidence

Generation completed within its 600-second bound: process wall time 36.755 s,
sampled peak RSS 981,200 KiB. The reported compilation component is 19.471 s.
Integration completed within 180 seconds: process wall time 124.493 s, artifact
loading 28.235 s, caller integration/reporting 95.912 s, sampled peak RSS
1,381,004 KiB. These are one-run diagnostics from the preserved executable, not
matched performance acceptance.

Every kernel received 1,024 points in each of eight complete shifts, seed
20261004, two workers and Korobov3. The checkpoint's native Kuo38005 rule has
modulus 1024 and generator `[1,309,235,573,145,523,153,653,577]`. Every sector has
the same native shift plan, preserving the common-shift interpretation. The
checkpoint contains exactly 5,592 canonical accepted packages, each covering
1,024 points without a failed prefix; starts cover `0,1024,...,7168` exactly once
per sector. Total accepted and planned kernel-points both equal **5,726,208**.

| Epsilon order | Mean | Standard error |
| --- | ---: | ---: |
| −2 | −4.067405136814634 | 0.28758494646617144 |
| −1 | −18.169409299577108 | 1.0692199476005346 |
| 0 | −162.1952374736587 | 8.959016118757578 |

The complete three-by-three covariance is retained. All means/covariances/errors
are finite, covariance symmetry holds, reported error squares match its diagonal,
and its principal minors are positive. The native saved-result reader accepts
the complete document. Report, snapshot and contributions have exactly the same
authoritative total; saved contributions, diagnostics and effective design agree
with the final report. No total was reconstructed by adding marginal variances.
The audit checks accepted checkpoint coverage, not an independent reimplementation
of its compensated estimator.

Diagnostics retain 1,232,525 conditioning checks, 1,160,471 rescues, 1,366 weighted
checks, 555 additional replays, maximum 320-bit precision and **zero evaluation
failures**. About 20.3% of accepted samples required rescue; this is an attribution
lead, not justification to disable guards. The older CLI emitted 2,796 status
rows totalling 281,938,057 bytes; its IO remains included in the recorded time.

## Limits

The result correctly retains `FullIntegral` scope, unverified validation status,
work-limit stopping, `converged=false`, and no stored reference. Its exact offset
is the zero vector, and every sector marginal uses all eight shifts. The rounded
historical target is not independently certified; neither proximity to it nor
finite complete native values establish correctness or calibrated uncertainty.
Independent-reference and repeated-seed/convergence evidence remain necessary.
No scientific or transport inconsistency was found in this bounded output audit.

## Subsequent independent-reference proposal review

The source-only [ordinary F-only reference proposal](hard-four-loop-reference-proposal.md)
keeps the exact original full-orthant density while placing the fixed positive
integer-power U factor in native `other_polynomials`. This is a supported native
input channel, not removal of U from the integrand. Installed
`decomposition/geometric.py` constructs the fan from `sector.cast`, transforms
every `sector.other` with the same monomial maps, and preserves that factor on
each cone. Installed `code_writer/make_package.py` subsequently refactorizes
every other polynomial and includes its extracted monomial in the subtraction
powers alongside the cast factors and Jacobian. U's sign and possible negative
mapped powers at infinity therefore remain part of native subtraction.

`geometric_infinity_no_primary` explicitly integrates the whole positive
orthant. Reusing the existing request/card/UF readers and their exponent helper,
with strict original-variable, measure, exponent, prefactor and source checks,
avoids a second parser or normalization implementation. The ordinary
`IntegralLibrary(together=True)` route computes uncertainty from each complete
sector sum; no disteval marginal-variance recombination is required. The
proposed one-CPU, 600-second shared deadline and 180-second numerical cap remain
diagnostic bounds. No concrete external runner or scientific execution was
reviewed in this proposal check, and independent coefficient evidence remains
pending until its full physical tuple and actual allocation are audited.

### Concrete runner source review, 2026-10-05

The independent review now includes
`output/probes/run_hard_four_loop_together.py`, SHA-256
`f2d2a98b9f16d02c79c2dbe6400049145a8bebed22b73141c37a4be44293e83b`.
It binds the original native/reference cards and U/F files, uses the existing
request and UF polynomial helpers, and checks all nine ordered variables,
zero measure powers, exponents `(1,0)` and `(-3,1)`, unit prefactor, full domain
and absence of a sector selection. The native make-package call decomposes F
and retains U as `other_polynomials`, as reviewed above.

The runner separates generation, guarded ordinary compilation and guarded
`IntegralLibrary(together=True)` evaluation. All stage watchdogs are clipped
to one 600-second process budget; the numerical stage additionally has a
180-second cap and passes that remaining deadline to the native provider.
It retains the original string tuple before conversion and selects physical
tuple member two, with no coefficient arithmetic or missing-value filling.
The prescribed 8311-point/32-shift native QMC settings and seed are explicit;
actual work remains unknown until the native output is inspected.

Fail-closed import controls precede generation, source/tool hashes and the
allowed/selected CPU are recorded, later failed stages remain explicit, and
postflight checks detect changed sources. No source blocker was found. This
was a read-only review: neither import controls nor scientific execution were
run by the reviewer. Package metadata, complete returned orders, ordinary
sector-sum uncertainty and actual completion remain outcome acceptance gates.

The subsequently retained controls in
`output/diagnostics/reference-import-controls-20261005T020026Z` were inspected
independently. All three processes exit zero with no timeout and empty stderr:
the negative control rejects Symbolica before module loading, and the projected
and hard-integral callers import only the ordinary IntegralLibrary/converter
under that fail-closed guard. The frozen source manifest still matches. These
controls establish the intended import separation; they do not execute package
generation, FORM/C++ construction or numerical integration.
