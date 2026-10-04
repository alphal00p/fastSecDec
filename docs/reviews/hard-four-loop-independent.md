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
