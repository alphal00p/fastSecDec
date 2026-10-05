# Bounded native sector-sum uncertainty attempt

The proposed attempt uses the existing ordinary constituent `IntegralLibrary`
with `together=True`, rather than modifying disteval's randomization or
reconstructing its variance. The API/source reasoning is in
`issue-one-reference-uncertainty-options.md`. The current issue-1 fixture remains
Unverified and unchanged until a complete new observation and independent
normalization/coverage/uncertainty-path review exist. A single new observation
would address the known omitted sector covariance, not establish general
uncertainty calibration.

Copy the entire completed `issue1-attempt-2/package` into a new
`issue1-together-attempt-1` directory, verify every file hash and timestamp-
preserving copy, and retain all earlier source/output/audit records. Work only
on the copy. The original constituent still has 616 sectors, seven integration
variables, all orders `[0,1,2]`, unit prefactor and unit outer sum coefficient.
It represents the full positive orthant of the exact supplied `F^(eps-2)`.
No partial fractioning, graph measure, sector selection, or prefactor
reconvolution is involved.

Use ordinary native make in the copied constituent directory:

```
taskset -c 0-7 make -j8 fsd_psd_four_loop_test_int_1_pysecdec_integral_pylink.so \
  FORMTHREADS=1 FORMOPT=2 PYTHON="REFERENCE_PYTHON IMPORT_GUARD"
```

The retained read-only dry run (`output/issue-one-together-make-dry-run.log`)
lists 2,469 C++ compilations: 2,464 sector/wrapper objects, three shared source
objects, the Python interface and its QMC template instance, followed by native
archive/link commands. It lists no FORM or export-sector command. The completed
distributed `distsrc` objects are not interchangeable with these ordinary
`src` objects. These are planned commands, not a prediction of elapsed time.

Propose **600 seconds for compile, 180 seconds for numerical execution and 800
seconds for the enclosing stages**, with the existing 30-GiB process-tree RSS
watchdog and its five-second interruption/kill grace. Check CPU0–7 are allowed
and eight distinct physical cores; native compilation has eight jobs and
FORM/OMP/BLAS one thread. Numerical execution uses CPU0 and one QMC worker. The
180-second numerical watchdog also covers loading and native result conversion,
and remains authoritative where ordinary pylink ignores amplitude-only deadline
settings. Preserve every partial package and failed outcome; no automatic
extension or retry is proposed.

Load the constituent `*_integral_pylink.so`, not the outer sum-package library.
Verify its public metadata before evaluating. Call existing `use_Qmc` with
Korobov3, no fit function, published `cbcpt_dn1_100`, `minn=8311`, `minm=32`,
`maxeval=265952`, explicit nonzero seed **20261218**, one CPU thread,
`standard_lattices=True`, zero lattice candidates, `epsrel=0.01`, and
`epsabs=1e-12`. These are a prespecified complete-allocation observation, not a
request to select favorable results. The native loop starts with the requested
lattice and shifts before testing its work limit (`qmc.hpp:3331–3342`);
actual returned N/R and all coefficient-call logs must nevertheless be audited.
It integrates each summed scalar Laurent coefficient, not the three-component
vector at once, so any reported work needs that unit and no joint
cross-coefficient covariance is invented.

Call `IntegralLibrary(..., together=True, format="series")` once with empty
physical parameter lists, no contour presampling and all compiled orders.
Persist the original three-member string tuple before invoking the library's
existing `series_to_json` converter. Member two remains the full physical
result. Capture native metadata, library hashes, exact settings, raw tuple,
conversion result/error and all logs. Conversion failure must not discard the
raw scientific output. The original source-level unit-prefactor proof is
rechecked on the compiled output before accepting any new result.

The ignored launcher draft is `output/probes/continue_issue_one_together.py`.
It reuses the already reviewed immutable-copy/watchdog helpers and the
fail-closed Symbolica import guard. Source hashes additionally cover native
`pylink_integral.hpp`, `qmc.hpp`, QMC wrapper and uncertainty headers. A guarded
import-only control must pass before allowing any overlap with a native
Symbolica experiment. Overlap would be diagnostic science on disjoint CPUs;
no timing or speedup acceptance follows. The draft is not a runtime approval.

On success, use the existing native reference writer/comparison APIs to retain
all orders and provider uncertainties. Keep the previous disteval observation
and its ineligible comparison as distinct evidence. Compare the new full vector
against the existing native result and, diagnostically, the old external means;
do not combine samples or estimate a missing covariance. Independent review
must verify that the actual compiled ordinary interface took its sector-sum
branch before considering a validation upgrade.

## Independent source review and execution boundary

The coordinator independently read the frozen launcher, immutable-copy/watchdog
helpers, generated constituent wrapper, installed `IntegralLibrary.__call__`,
ordinary `pylink_integral.hpp` sum branch and native QMC loop. The wrapper includes
the ordinary integral header; the `together` branch sums all sector integrands
before native integration, and the QMC loop performs its initial allocation
before its work-limit check. The public call accepts the recorded settings.
Both the fail-closed Symbolica rejection control and guarded import of the
existing ordinary library/converter passed. Their logs are retained as
`output/issue-one-together-{guard,import}-control.log`.

The coordinator authorizes the single fixed 600/180/800-second attempt described
above, with no performance acceptance or automatic extension. Native compiler
and source hashes, unchanged parent, returned complete physical tuple, actual
N/R and the compiled sum branch still require outcome review. The current
Unverified fixture and its errors remain unchanged by this authorization.
