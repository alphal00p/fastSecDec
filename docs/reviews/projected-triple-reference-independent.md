# Independent projected triple-box reference runner review

This is a source-only review of
`output/probes/run_projected_triple_together.py` against the
[prescribed attempt](projected-triple-reference-attempt.md), the installed
pySecDec interfaces and the previously audited positive-power projection.
No new reference value or numerical process is accepted by this review.

The reference reads the original literal propagators, momentum bases, numerator
and kinematics with the existing reference readers. It supplies the proved
coefficient-one original-order power vector `[1,1,0,2,0,2,1,1,1,1]` to native
`LoopIntegralFromPropagators`; that implementation removes zero-power entries
and owns the Gaussian numerator, raised-power measure and Gamma factors.
The runner checks the resulting eight parameters and active powers
`[1,1,2,2,1,1,1,1]`. Scalar and rank-two observations use separate output
directories and seeds. Neither receives a FastSecDec U/F polynomial, coefficient
or numerical result.

The installed public `loop_package` documents the selected
`package_generator=pySecDec.code_writer.make_package` route. Its `LoopPackage`
adapter passes the native measure and numerator, with the native Gamma factor
multiplied by the explicit additional prefactor one. This creates an ordinary
constituent library. Native `IntegralLibrary(together=True)` sums sector
integrands before QMC estimates each scalar Laurent coefficient. The complete
physical tuple member two already includes its prefactor; the runner preserves
the original tuple before conversion and applies no additional convolution or
zero filling. Actual prefactor, complete order coverage, imaginary components,
allocation counts and errors still require outcome review.

The explicit QMC arguments are supported by the installed ordinary interface:
Korobov3, no fit function, `cbcpt_dn1_100`, N8311/R32, one numerical thread,
separate nonzero seeds, standard lattices and zero candidate extension. The
runner correctly calls these requested settings, leaving actual work unknown.
The native `requested_orders` library metadata key exists and is checked in
addition to regulator and parameter layout. A positive sector count prevents a
vacuous empty allocation.

Generation, compile and numerical stages are separate subprocesses under one
600-second deadline; each receives a clipped remaining budget, and the numerical
stage also has a 180-second cap. The reviewed watchdog enforces the 30-GiB
process-tree RSS bound and preserves stage commands, partial packages and
nonzero outcomes. One allowed CPU is explicitly selected. Generation needs the
exclusive Symbolica slot; subsequent native make helpers and numerical loading
use the existing fail-closed import guard. The guarded ordinary-library import
control and runtime handoff remain execution prerequisites. There is no new
license setting or guard bypass.

The recording is immutable and binds original scientific inputs, provider
sources, interpreter, compiler tools and generated output hashes. Review
requested inclusion of native `loop_integral/common.py` and
`secdecutil/uncertainties.hpp`, the delegated measure/Gamma and uncertainty
owners. Both are included in the frozen runner, SHA-256
`c14bb4faef75a026d6d09ba83a8d836dd1aea3920e6140043552c6beabb63beb`.
Source checks after execution must remain successful. A generated or compiled
package alone is not an independent
integral comparison, and a completed low-statistics reference alone is not a
convergence or performance certificate.

## Scalar attempt 1: generated input accepted, numerical result unavailable

The retained attempt under
`output/diagnostics/remaining-pysecdec/triple-scalar-together-attempt-1`
completed native generation in 136.321468297 seconds, then hit its original
shared 600-second deadline during guarded compilation. The outer watchdog
returned 124 after 601.680116823 seconds including termination. Its retained
process-group check is empty, and no numerical stage, tuple or result exists.
This is a bounded incomplete reference attempt, not a failed integral check.

Independent inspection confirms the original ten propagators and coefficient-one
power projection, the eight active powers `[1,1,2,2,1,1,1,1]`, scalar numerator
one, and the prescribed off-shell scalar products. Native Gamma ownership is
`Gamma(4+3 eps)`: the generated prefactor starts at six and its displayed linear
coefficient is `33-18 EulerGamma`, consistent with that factor. No extra
prefactor is applied by the reviewed ordinary-library caller.

The generated ordinary package has 1,182 sectors and 1,946 coefficient kernels:
68 at order -3, 206 at -2, 490 at -1 and 1,182 at zero. These package-layout
counts are not completed QMC work or a native/reference sector correspondence.
The package has seven integration coordinates, requested maximum order zero
and no remaining numerical parameters. Physical tuple/order/errors must still
be reviewed after a separately authorized successful continuation.

All 29 recorded source hashes and all 4,313 retained package-file hashes were
rechecked successfully after termination. The package contains 136 completed
FORM stamps and 412 object files; it is an immutable parent for any later
copied-package attempt. The final package manifest SHA-256 is
`dfd9082ca6cf1055cf8fd30c409a754e3d22a3086e2c539cfb758e837733fdbc`,
and the retained outcome SHA-256 is
`b8ddeb56eba6b978f146fc5cadf7f6c792602d1e6e3da4bffa310f9707b59e72`.
No automatic deadline extension or numerical certification is accepted here.

## Copied-package continuation source review

The separate proposed continuation
`output/probes/continue_projected_triple_together.py`, SHA-256
`42329c44c858a48761089c08cd7b4409bb512c82aeb349a1d88e22c10200d214`,
passes source review. It admits only the retained generated-but-uncompiled
scalar timeout with no prior numerical stage, verifies source/tool identities,
and compares every copied package byte and modification time with the immutable
parent. The ordinary generated Makefile owns its sector prerequisites and
library linking; plain `make -j8` adds no source edits, stamp overrides or
replacement dependency logic.

Its proposed 1,200-second compile and 180-second numerical caps are clipped by
a 1,400-second outer deadline, with the existing 30-GiB process-tree RSS bound.
The eight compile CPUs are checked for availability and distinct physical
cores. Both guard controls repeat before native make, whose Python helper
commands remain guarded. The one-CPU numerical stage invokes the unchanged
frozen scalar `integrate` function, retaining together=True, the original
seed/transform/lattice and the full original physical tuple. Parent contents,
timestamps, records and sources are checked after termination.

This review permits no cold-generation timing, eight-core integration or
speedup claim. Execution requires coordinator scheduling; actual copy checks,
guard outcomes, loaded package layout, prefactor and numerical errors remain
separate outcome gates. The source review itself launched no process.

## Completed scalar reference and uncertainty audit

Attempt 2 preserved its successful 324.765602353-second ordinary-library build
and its separate 180-second numerical timeout. No partial coefficient log was
promoted. The separately authorized attempt 3 copied that completed package,
kept the original scientific caller and seed, and completed all four coefficient
calls. Its numerical process took 233.887371854 seconds under the 750-second
cap; the complete continuation took 237.626191778 seconds under 770 seconds.
The original ordinary `wall_clock_limit=180` argument is explicitly unused by
`pylink_integral.hpp`; the external watchdog bounds are authoritative.

Independent rechecks passed for 31 source files, 38 parent records, 27 ancestor
records, and every file in both the 16,845-file parent and copied packages.
The current copied manifest equals the initial copy manifest. Both guard
controls and both process records return zero, the recorded process groups are
reaped, and all parent/copy/source postchecks pass. The compiled library remains
SHA-256 `299ea47a01e76bf71f0562408cfec4d64261ede3b03e85032ed9499e49aff0d2`.

The raw native string tuple is retained before `series_to_json`. Physical tuple
index 2 contains the original integral times native `Gamma(4+3 eps)` exactly
once. Its complete physical coefficients are:

| Order | Value | Reported standard error |
| --- | ---: | ---: |
| -3 | -9.85658834056313826e-8 | 1.50731205992427189e-7 |
| -2 | -0.246844089618439316 | 0.00325092179836941999 |
| -1 | 0.773566283495650597 | 0.0163022154681442093 |
| 0 | 2.28341348840049418 | 0.0571853793808127786 |

Every imaginary value and imaginary error is exactly zero in the native tuple
and its converted binary64 tags. The independent transport check compares all
raw max-digits-10 decimals with those tags and retains exact hex/bit identities.
No prefactor is reapplied, no uncertainty becomes exact, and no covariance is
invented. An initial audit JSON draft exposed Perl's short default numeric
serialization; it was corrected before native fixture consumption using the
unchanged raw decimal strings and exact binary64 identities.

Source inspection confirms that ordinary `together=True` adds all sector
integrands before each coefficient's QMC call. The native integrator captures
one QMC object, seeded once with 20261201, and advances its random generator
across coefficient calls. This avoids the previously identified disteval
per-kernel shared-shift/marginal-variance issue. Native uncertainty arithmetic
then propagates the independent coefficient errors through the deterministic
Gamma series. Physical orders become correlated through that convolution;
their joint covariance is not supplied and must remain unknown. This review
accepts the native reported marginal errors without certifying their empirical
calibration.

The log records four completed calls, each with 8,311 lattice points and 32
shifts, and each explicitly reports 265,952 evaluations. Their total 1,063,808
counts **scalar summed-sector coefficient points**, excluding setup and
auxiliary work; it is not a full-vector sample count. The package's 1,182-sector
layout is not a sector correspondence with FastSecDec. Loading was separately
reported as 0.026469213 seconds and the native call as 233.126451885 seconds;
these are correctness diagnostics, not matched performance measurements.

`independent-review.json` in the attempt-3 directory recommends `Checked`
reference transport with explicit source/kinematics/normalization evidence,
reported `StandardError`, unknown cross-order covariance, and
`calibration_certified=false`. The finite error is approximately 2.5%, so this
does not meet the requested one-per-mille accuracy goal. The near-zero leading
coefficient remains its observed value and error, not an exact zero. Native
comparisons against the separately retained original and projected full vectors
are the next transport gate; this outcome audit does not anticipate their
results.

## Frozen native transport gate

The native writer `output/probes/record_projected_triple_scalar.rs` uses the
existing `ReferenceResult`, encoder/reader and comparison APIs. Its audit input
is SHA-256 `efda8b3859e6288cb28ebedf876ec762a7fb2621d9ff4087b7d242bf56b4288d`.
All eight transported mean/error binary64 identities independently match the
provider's raw decimal/hex/bit record. The retained original and projected
native estimates exactly match their source records, including all 16 entries
of each native covariance matrix; no estimates are recomputed or pooled.

The two native comparisons preserve complete orders `[-3,-2,-1,0]`. Absolute
pulls are at most 2.8151 for the original representation and 2.0220 for the
projected representation. Both use the same audited external observation, so
these are separate diagnostic comparisons, not two independent reference
measurements. Their native seeds may correlate the original and projected
estimates with one another. `Checked` removes only the reference's unverified
status after the scoped scientific audit; calibration, exactness and the
requested tolerance target remain explicitly false.

The frozen fixture `examples/references/triple_box_offshell_scalar.json` is
byte-identical to the native writer output, SHA-256
`0129638e3a8913b2e9abc3542fb1ca852d00fe47dd4951c1e017e9d9e8763c0c`.
The focused `multiloop_references` target passes five active tests with two
ignored recording/replay tests, including the new exact-bit full-vector,
measure, imaginary-projection, source and covariance-availability assertions
(`output/projected-triple-scalar-reference-tests.log`). No new scientific
sampling was performed by this transport gate or independent review.
