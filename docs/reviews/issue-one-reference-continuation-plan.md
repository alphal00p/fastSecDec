# Issue-1 copied-package continuation: source review

This is a conditional plan, not an extension of the active first attempt. That
attempt retains its 600-second bound. No continuation is launched without a
separate coordinator decision after its final outcome and retained work audit.

The external generator has already produced the full seven-dimensional
positive-orthant package: 616 sectors and 1,848 coefficient kernels spanning
orders `[0,1,2]`, with unit prefactor. Current work is native FORM source export
and C++ compilation. The original input, subtraction/decomposition policy,
coefficients and numerical request must not change to finish this build.

The native generated Makefiles support a bounded parallel build:

- The sum-package `Makefile:72` delegates to `$(MAKE) -C INTEGRAL disteval.done`,
  so GNU make's jobserver carries the outer `-j8` limit into the submake.
- Integral `Makefile:45` makes each `codegen/sectorN.done` depend on its own
  `sectorN.h`. Its installed `formwrapper` invokes FORM with the sector ID;
  `export_sector` writes that sector's C++ sources before touching the stamp.
  Each `codegen/sectorN.d` declares all that sector's generated files dependent
  on the same completed stamp. No timestamp override or forced rebuild is needed.
- Each coefficient object depends on its own generated C++ source. The final
  integral shared-library link depends on all required objects; the sum package
  links the resulting library into its native disteval directory. Use ordinary
  `make -j8 disteval FORMTHREADS=1 FORMOPT=2`, preserving C++ flags and using eight
  distinct physical cores. This is eight independent jobs, not eight FORM
  threads per job.
- The installed `formwrapper` imports only standard-library modules
  (`os,re,subprocess,sys,tempfile`); `export_sector` likewise imports only the
  standard library. Neither imports FSD, pySecDec's symbolic package generator
  or Symbolica. `write_integrand.frm` writes `sectorN.info` using the explicit
  sector ID. Native `formwrapper` already uses an atomic replacement if it must
  enlarge shared `form.set`; leave this native behavior unchanged and retain any
  such event. Initial observed process-tree RSS near 0.2 GiB is only an
  observation, not a bound for eight simultaneous jobs.

After a failed first attempt has fully stopped, copy its entire generated package
to a new attempt directory, hash every original file and verify the copy before
make. Preserve timestamps and all earlier logs/outcomes. Retain an ordinary
make dry-run for pending targets, but do not mistake empty generated-file
recipes for actual recompilation counts. Never edit source integrands, dependency
stamps, normalization or the parent package. Recheck parent hashes afterward.
Bind the existing compiler/FORM/make/helper paths, versions and hashes.

For a concrete continuation, propose a 600-second compile bound, 180-second
numerical bound and 800-second enclosing bound, all under the existing 30-GiB
process-tree memory watchdog. CPU0–7 must pass the existing physical-core/allowed
affinity preflight; keep FORM/OMP/BLAS threads at one. Record the exact remaining
stamps/objects when the first attempt ends before approving these limits. These
are conservative operational budgets, not predicted completion times or speedup
measurements.

The numerical step can use the installed public `pySecDec.disteval` CLI directly
on the completed sum-package JSON. Its `main` at `disteval.py:1020` delegates to
the same `prepare_eval`/`do_eval` as the current FSD bridge. Preserve the original
settings explicitly: `--points=8192 --presamples=8192 --shifts=32`,
`--epsrel=0.01 --epsabs=0 --standard-lattices=yes --lattice-candidates=0`,
`--timeout=180s --format=json`. A recorded cluster file contains exactly one
existing `pysecdec_cpuworker` command, and numerical execution returns to one
allowed CPU. The bridge supplies the same arguments, including standard-lattice
selection and zero lattice candidates; there are no external parameter values
for this fixture. Keep all adaptive kernel-work logs and the complete native
JSON output before any comparison or order alignment. The initial N/R are not
final observed work, and this direct CLI does not provide a new covariance.

A tiny external-only import guard may execute the unchanged installed scripts
or module while rejecting any Symbolica import; it must preserve argv and have
its own source hash and meaningful guard control before use. Passing it through
the native `PYTHON` make override also covers FORM/export helper interpreters.
This is an audit guard, not a licensing alteration. No FSD import, symbol
re-registration, worker-instance workaround or scientific parser replacement is
needed. Independent review of the actual launcher, copied evidence and final
all-order result remains required.

After the original attempt's retained timeout, the coordinator approved this
separate continuation and the independent reviewer checked the concrete
`continue_issue_one_reference.py` and `guard_reference_python.py`. Guard controls
reject Symbolica before loading, successfully start the unchanged native
disteval CLI and FORM helper, and reproduce all six sector-1 C++/CUDA exports
byte-for-byte in a separate control directory. The launcher also hashes the
actual Python executable, `pySecDecContrib.__main__` and native CPU-worker binary.
These checks are recorded in `output/diagnostics/issue1-continuation-guard`.

Attempt 2 then verified and copied all 6,752 parent files (42,329,285 summed file
bytes) before starting native make. Its preparation confirms eight distinct
allowed physical cores on CPU0–7 and one later numerical worker on CPU0. Evidence
is under `output/diagnostics/remaining-pysecdec/issue1-attempt-2`; the new bounds
are exactly those above. This records a separately authorized start, not a
successful numerical outcome. The original attempt remains unchanged.

The subsequent attempt completed: native make took 81.301481 seconds, the
numerical subprocess 51.586893 seconds and the enclosing process 133.994916
seconds, all with exit zero. Parent and guarded-source hashes still match.
Physical sum orders `[0,1,2]` and their positive reported standard errors are
retained without reconstruction in `native-result.json`; the original stdout
and constituent rows remain preserved. Independent review confirms complete
1,848-by-32-by-8,311 integration coverage, excluding presampling/calibration.
The scientific values and remaining interpretation limits are recorded in
`remaining-reference-attempts.md`. No bound was extended and the first attempt's
timeout remains a separate observation.
