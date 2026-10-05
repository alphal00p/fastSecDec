# Native options for the issue-1 reference uncertainty

This is a read-only audit of the installed pySecDec 1.6.6 sources and the
completed issue-1 package. It proposes no new estimator, dependency patch or
numerical execution. The returned physical means and reported errors remain
unchanged in the Unverified fixture.

The current disteval public interfaces do not expose independent per-kernel
seeds. `pySecDec/disteval.py:430` accepts prepared workers, tolerances, counts,
lattice options, parameter values and a deadline; line 556 creates one
`numpy.random.RandomState(0)` per kernel internally. The CLI and public
`DistevalLibrary.__call__` (`integral_interface.py:1661–1707`) forward no seed
parameter. Equal-dimensional issue-1 kernels therefore consume matching shift
sequences at the same lattice count. Lines 757–782 compute marginal kernel
variances and multiply them by squared weights; they do not use cross-kernel
covariance. An input option cannot correct this in the inspected version.

An existing ordinary-integral API can instead estimate the sum before computing
its uncertainty. `IntegralLibrary.__call__(together=True)` is documented at
`integral_interface.py:948` and defaults to true at line 1286. Native
`secdecutil/pylink_integral.hpp:163–175` uses `std::accumulate` on all sector
integrands and then applies the existing integrator to each Laurent coefficient
of the combined integrand. Native QMC therefore observes the entire sector sum
per sample, including its cancellations and correlations; there is no external
variance reconstruction. This still does not return the full joint covariance
between different Laurent coefficients and does not itself establish error
calibration across independent runs.

The distinction between a constituent and a sum package matters. The sum-package
`pylink_amplitude.hpp` accepts the `together` argument but constructs amplitude
objects without using it to merge sectors. Its generated weighted-integral
wrapper creates separate native integral objects. Simply loading the outer
sum-package library with `together=True` would not prove the desired behavior.

For this particular issue-1 package there is exactly one constituent, with unit
sum coefficient and unit prefactor. Its already generated
`fsd_psd_four_loop_test_int_1_pysecdec_integral/pylink/pylink.cpp:21` includes
`secdecutil/pylink_integral.hpp`; thus loading that constituent's ordinary
`*_pylink.so` directly provides the supported together route without changing
the density, sectors or measure. Public `use_Qmc` exposes an explicit nonzero
seed (`integral_interface.py:685–686`), and generated `pylink.cpp:359–360` passes
it to the native random generator. A future separately reviewed attempt can
use an explicit seed distinct from the retained native integration, all three
orders, one numerical worker and the existing import guard.

The build is additional work, not a free reuse of the completed disteval binary.
The constituent Makefile's ordinary `*_pylink.so` target depends on
`pylink/pylink.o`, `libNAME.a`, and QMC template objects. That archive uses
`src/*.o`, while the completed distributed library uses `distsrc/*.o`. Original
generated sources and FORM stamps are already present, but ordinary C++ objects
and wrapper templates still need compilation. Any attempt must copy and verify
the completed parent package, retain the earlier outcomes, and separately bound
native build and numerical work. No automatic rerun, deadline extension,
source modification, or reference validation upgrade follows from this source
audit alone.

The same installed disteval policy warrants a separate review of other external
reference fixtures that used it. Matching dimensions, lattice counts and shift
histories must be inspected before claiming independence of their sector errors.
The current issue-1 decision must not silently reinterpret or rewrite those
earlier numerical observations.
