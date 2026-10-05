# Independent remaining-reference orchestration audit

This is a source/API review of the ignored launchers
`output/probes/run_remaining_reference.py`,
`capture_pysecdec_disteval.py`, `capture_pysecdec_series.py`, and
`projected_triple_reference.py`, against the frozen Pathfinder revision and
installed pySecDec 1.6.6. The follow-up issue-1, hard-orthant and projected
triple-box launchers had not been executed when reviewed. This review starts no
reference process and establishes no numerical reference result.

No blocking scientific input or normalization mismatch was found:

- The projected triple-box caller uses the existing reference DOT and kinematics
  readers, preserving the original loop/external basis and numerator. The two
  exactly repeated propagator pairs at positions 2/3 and 4/5 admit the recorded
  coefficient-one power replacement. Native
  `LoopIntegralFromPropagators(..., powerlist=...)` removes zero-power lines and
  their parameters itself (`loop_integral/from_propagators.py:166`). The caller
  checks the resulting active powers `[1,1,2,2,1,1,1,1]`.
- `kinematics.py:49` supplies fixed FORM-compatible numeric replacement strings,
  including the all-external-virtualities-minus-one and s12=s23=-2 point. The
  direct constructor therefore needs no additional numeric real parameters.
  Scalar mode changes only the numerator to one; rank-two mode preserves the
  supplied contracted expression.
- Native `LoopPackage` inserts `Gamma_factor` exactly once together with the
  raised-power measure (`loop_package.py:80–96`). No projected Jacobian, factorial
  or extra Gamma convolution is introduced. The direct caller retains the entire
  `IntegralLibrary(..., format="json")` tuple and identifies tuple member 2 as
  the full physical result, matching `integral_interface.py:1353–1368`.
- The U/F caller uses the existing `run_pysecdec_uf_all_sectors`, which rejects a
  sector filter and delegates decomposition to native make_package. Both cards
  retain `geometric_infinity_no_primary`, so the domain is the full positive
  orthant. The supplied global prefactors are one; the known generic U/F bridge
  prefactor omission is therefore inactive for these two inputs only.
- The native disteval lane receives initial points/shifts and a numerical
  deadline from `_run_pysecdec_disteval`, rather than an IntegralLibrary maxeval
  budget. The wrapper captures its native result before delegating to the
  unchanged parser. It neither adds absent orders nor sets missing uncertainty
  to zero. The original tuple capture similarly precedes parser invocation, so a
  parser exception cannot destroy already written evidence.

The orchestrator requires a new absolute output directory, records exact argv,
source revision/status/hashes and compiler/package versions, fixes one allowed
CPU and native generation threads, and uses the existing 600-second/30-GiB
process-tree watchdog. The direct propagator lane additionally passes
`processes=1` explicitly. The documented command uses the frozen reference
environment's interpreter, which matters because preparation package versions
are collected in the launcher process.

These are bounded correctness attempts, not matched performance measurements.
Native observed work, actual generated package prefactors, complete order
coverage, finite means/errors and the native full-result tuple must still be
checked for every successful attempt. Initial lattice requests and maxeval are
not observed evaluation counts. Raw imaginary values/errors must be retained
before any justified projection to real-only FastSecDec outputs. A timeout,
package-generation failure or successful process with no complete physical
result leaves the independent reference gate unresolved.

The already launched original double-box attempt was reported reaped after its
600-second bound with no numerical tuple. That is retained failed-attempt
evidence, not a reason to accept a historical placeholder target or alter any
native FastSecDec normalization.

## Copied-package double-box continuation

The separately authorized attempt-2 continuation was also reviewed from
`output/probes/continue_double_box_reference.py` and its actual preparation and
command records. All three parent-record hashes, six guarded-source hashes and
1,766 original package hashes (45,317,063 bytes) matched during independent
inspection. The active build works on a verified copy; the failed parent package
was unchanged. `FORMTHREADS=1` and `FORMOPT=2` match the original native Makefile
defaults and introduce no generated-integrand change.

The actual nested watchdog commands bind compile/numerical/outer limits of
1800/180/2000 seconds and 30 GiB, with the numerical child using the existing
native IntegralLibrary, an explicit seed of 20261203, the checked `s12 s23`
parameter layout at `[-1,-1]`, and physical tuple member 2. A guarded import
finder rejects Symbolica imports; this continuation does not import FSD or
regenerate symbolic input. Concurrent independent diagnostics are disclosed,
so no matched timing claim is made.

The outer 180-second numerical child deadline includes library loading as well
as the native integration call; it can therefore interrupt before the latter's
own 180-second deadline. Any such incomplete outcome remains a retained timeout,
not a complete result. The full native tuple, nonzero errors, finite physical
coefficients and observed coverage still require a separate outcome review.
## Completed double-box continuation: attempt 3

The third attempt successfully completed the existing native package with
ordinary `make -j1 pylink FORMTHREADS=1 FORMOPT=2`. There was no `make -o`
override, regenerated integrand, Symbolica import, or changed normalization.
The source differs from the previously reviewed continuation only in its
approved 300/180/500-second compile/numerical/outer bounds and the exact parent
evidence filenames. A prior `make -n` prediction did not establish which object
files would actually be rebuilt; the retained ordinary make execution is the
relevant evidence.

Independent verification checked all ten parent records, six guarded source
files, all 2,097 parent package files (77,406,484 bytes), and five final
library/prefactor/configuration hashes. The parent package remains unchanged.
Compile, numerical subprocess and outer process exited zero in approximately
30.849, 48.878 and 80.836 seconds, respectively. These shared-host continuation
costs are diagnostic observations, not comparable generation benchmarks.

The native library reports real parameters `s12 s23`, supplied as `[-1,-1]`,
seed 20261203, one numerical thread, Korobov3, relative tolerance 0.01 and
absolute tolerance `1e-7`. The tagged result decodes exactly to the independently
retained raw tuple. Its physical third member equals the first member; the
outer sum-package prefactor is `1`, while the unchanged constituent source
already supplies `-Gamma(3+2*eps)` through the required order. Applying another
Gamma prefactor would therefore be incorrect.

| epsilon order | Physical real mean | Reported standard error |
|---:|---:|---:|
| -4 | -1.0269562977782698e-15 | 2.1150505867918003e-15 |
| -3 | 1.500432835114506 | 0.00036128998422068834 |
| -2 | 1.269034715961728 | 0.0016102860609434047 |
| -1 | 2.997315028586949 | 0.006481159774516808 |
| 0 | -14.883187577966803 | 0.027253699339267728 |

All values and errors are finite; all reported imaginary means and errors are
zero. The tiny leading-pole error remains a reported standard error, not an
exactness assertion. No covariance matrix is supplied by this transport, and
none is invented. Verbose logs show native calls with 10,061 points and 32
shifts, exceeding the requested `maxeval` value; the aggregate actual point
count has not been independently reconstructed and remains unknown.

The full audit record is
`output/diagnostics/remaining-pysecdec/double-box-attempt-3/independent-review.json`.
This is suitable independent low-statistics correctness evidence after the
native comparison adapter checks the matching integral convention. It does not
alone certify convergence, performance parity or all remaining examples.

The native reference recorder and `examples/references/double_box.json` were
then reviewed independently. All five serialized values and strictly positive
`StandardError` uncertainties exactly match the audited physical tuple. The
real-only projection retains the zero imaginary means/errors in provenance;
the tiny measured leading pole is preserved. Source hashes, the copied-package
manifest record digest, parameter ordering, prefactor ownership and earlier
failed attempts remain available. Unknown aggregate reference work and joint
covariance stay unknown.

The recorder uses the existing native `ReferenceResult` encoder/reader and
`reference::compare`. Its native estimate, including the full covariance matrix,
is byte-for-value unchanged from the completed 64-shift report. Marking the
reference Checked changes eligibility, not the numerical comparison rows.
The five native combined-error pulls are approximately
`[-0.3075, 0.2081, -0.1691, 0.9673, 1.2652]`. The checked label records this
independent implementation/input/transport evidence; it does not certify the
uncertainty model or convert sampling results into analytic truth. The frozen
transport test gate reports two passing tests, with two expensive/recording
tests intentionally ignored. No blocker was found for this scoped fixture.
