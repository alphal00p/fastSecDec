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
