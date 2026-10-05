# Bounded independent-reference attempts

## Double-box, first attempt

The first independent pySecDec DOT attempt ended at the prescribed **600-second
wall limit during code generation**, before numerical integration. It produced
no reference value, uncertainty or returned tuple. Higher double-box Laurent
coefficients remain independently unverified; the already established exact
leading-pole cancellation is a separate check.

The unmodified frozen Pathfinder native-DOT engine used the actual shipped
seven-propagator graph at the massless on-shell point `s12=s23=-1`, requested
orders through zero, the normalized `d^Dk/(i*pi^(D/2))` measure, iterative native
pySecDec decomposition, one allowed CPU and one native thread. No historical
manual target was passed. The approved command and preservation contract are in
[the reference protocol](remaining-reference-protocol.md).

The generated package contains 96 native sectors, lowest/highest integrand orders
minus four through zero, and prefactor orders zero through four. Its constituent
`prefactor.cpp` contains the expansion of `-Gamma(3+2*eps)`; the sum-package's
weighted-integral implementation explicitly multiplies that constituent
prefactor. Those sources and disteval metadata are retained with hashes. This
establishes how a future successful returned tuple must be normalized; it is not
a numerical result and no second Gamma convolution has been performed.

FORM sector 85 required 123.31 seconds on its first code-generation pass, exhausted
the configured `WorkSpace=150000000`, and automatically retried with
`WorkSpace=450000000`. That retry was still active at the bound. The reference's
existing watchdog observed 600.2 seconds, sent SIGINT to its own five-process
tree, and recorded that the tree exited after SIGINT. The wrapper returned 124
and completed at 600.999 seconds including its metadata/finalization. Peak
sampled process-tree RSS was 0.634 GiB; the 30-GiB cap was not reached. The frozen
program's generic “interrupted by user” text refers here to the watchdog's
interrupt, not a user cancellation.

Evidence is retained at
`output/diagnostics/remaining-pysecdec/double-box-attempt-1/`: exact command,
source/package/environment hashes, watchdog and generation logs, completion
record, and the generated/partially compiled package. There is no `result.json`,
no shared pylink library and no raw-series file. The recorded child PID was
absent after completion. Workspace compilation overlapped with explicit
coordination; no scientific test shared the Symbolica runtime. These
elapsed times are diagnostic and cannot establish matched performance parity.

The cheapest proposed continuation is a **separate recorded attempt** that copies
the retained package to a new output directory, verifies its source/package
hashes and resumes native `make -j1 pylink`, then invokes the existing native
`IntegralLibrary` with a new explicit seed and a separate numerical wall bound.
Do not rerun the frozen CLI in a way that deletes the incomplete package, change
the integrand or decomposition, or overwrite the failed attempt. The generated
Makefiles use relative targets and `CURDIR`; verify any recorded absolute paths
before copying. The subsequently approved continuation is recorded below.
Issue-1 and the other prepared commands remain queued for coordinated runtime
slots.

### Concrete continuation proposal

Read-only inspection of the retained package finds **83 completed sector
code-generation markers and 324 compiled constituent objects**. Thirteen native
sectors remain: **9 and 85–96**. Sector 9 is still pending because Make traverses
the generated target list in its existing order; the highest completed sector
number alone would incorrectly imply only twelve remaining. Outer pylink,
amplitude and weighted-integral objects are already compiled, while the final
constituent archive, QMC template objects and shared-library link remain.

The difficult sectors are visible in the generated inputs: sector 85/86 headers
are about 1.92 MB each and sector 87/88 headers about 3.21 MB each. These sizes are
not runtime estimates, but rule out treating another unmodified 600-second full
restart as an evidence-based completion plan. The retained `codegen/form.set`
already records `WorkSpace 450000000` after native `formwrapper` increased it.
The native wrapper triples workspace only when FORM explicitly reports overflow;
it does not alter the integral. `Makefile.conf` fixes one FORM thread, native FORM
optimization level two and C++ `-O2`. Preserve those settings initially.

Propose one new attempt with **1800 seconds for remaining native source/compile
work**, **180 seconds for numerical integration**, and a **2000-second outer
process-tree bound**, retaining the existing 30-GiB memory limit. The extra
budget is reserved for the thirteen known pending sectors and final linking,
rather than repeating completed generation. Before execution:

1. Copy the entire package to a new attempt directory with timestamps preserved;
   record the first attempt as its parent and SHA-256 every copied source,
   Makefile, completed object and `form.set`. Do not mutate the failed attempt.
   Generated `TOPDIR` is derived from the current Makefile location; fixed Python
   and pySecDecContrib paths point to the frozen external environment, not the
   first output directory.
2. Run the native `make -j1 pylink FORMTHREADS=1 FORMOPT=2` inside the copy under
   its compile-stage watchdog. Retain the native automatic workspace-growth
   log and memory observations. A second bound/failure remains a failed attempt;
   do not silently lower FORM optimization or change the density.
3. Only on successful linking, load the copied native shared library and call
   `use_Qmc(seed=20261203, cputhreads=1, verbosity=1)` and the unchanged full-vector
   `IntegralLibrary` interface with `epsrel=0.01`, `maxeval=65536`,
   `wall_clock_limit=180`, `number_of_threads=1`, `format="json"`. Preserve the
   entire native tuple before interpretation, all native logs and actual work
   where observable. Keep the standard absolute-error target explicit in the
   invocation record. The finite maximum-evaluation request is steering, not
   guaranteed realized work or accuracy.

This continuation is a proposed bounded scientific reference calculation, not a
performance benchmark or permission to overwrite the failed block. It requires
the coordinator's runtime allocation and approval of the changed bound before
launch; that approval and the ensuing attempt are recorded below. Native
FastSecDec production code and its integration settings are
unaffected.

## Double-box, approved copied-package continuation (timed out)

The coordinator approved that changed budget and the direct continuation has
started in `double-box-attempt-2/`. The ignored launcher is
`output/probes/continue_double_box_reference.py`. Before native make, it copied
and hash-checked all **1,766 package files (45,317,063 bytes)** from the immutable
first attempt and retained the hashes of its preparation, completion and watchdog
records. Independent source/preparation review confirmed all parent and launcher
hashes, the actual command bounds and unchanged native FORM/C++ settings.

This path imports no FSD module. A guarded import probe installed a Python
meta-path finder that rejects any attempted `symbolica` import, then successfully
imported `pySecDec.integral_interface.IntegralLibrary` with no Symbolica module
loaded. The same guard runs in continuation orchestration and numerical stages.
Native make invokes FORM/C++ and the installed standard-library-only
`formwrapper`/`export_sector` tools. Its CPU-0 execution may therefore overlap the
separate symbolic correctness proof on another CPU, with explicit coordination.
These are scientific diagnostics, not comparable timing runs.

The stage watchdogs retain the approved 1800-second compile and 180-second
numerical bounds, beneath a 2000-second outer watchdog, all with a 30-GiB
process-tree limit. The numerical stage's outer bound includes loading and
result capture, so practical integration may receive less than the native
`wall_clock_limit=180`; any resulting timeout is retained honestly. Native seed
20261203, the full tuple, physical tuple index two, normalization sources and
actual work where observable remain required before reference acceptance.
The compile watchdog reached **1800.3 seconds** while C++ compiled
`src/sector_95_0.o`. It interrupted its own process tree, which exited after
SIGINT. The recorded compile-process duration was 1801.562 seconds, including
watchdog shutdown; outer duration was 1802.455 seconds. Sampled peak aggregate
RSS was **0.577 GiB**. The parent package remained hash-identical. No archive or
shared library existed, the numerical stage was not started, and no coefficient
or uncertainty was produced. The process was reaped with exit 124.

This is a second retained failed reference attempt. It does not change the
independent status of the native double-box higher coefficients.

## Audited remaining work and third bounded continuation

The immutable second package contains **2,097 files / 77,406,484 bytes**, 95
completed FORM/export stamps and 395 retained object files. A timestamp audit
finds no existing object older than its corresponding C++ source. The missing
existing-source objects are the finite coefficient of sector 95, three common
constituent sources (`integrands`, `pole_structures`, `prefactor`), and the outer
QMC template. Sector 96 still needs native FORM/export and five C++ objects.
Its input header is 73,794 bytes, versus about 1.92 MB for sectors 85/86 and
3.21 MB for 87/88. Source size is only a cost indication, not a promised runtime.
The final constituent archive and shared-library link remain.

The first plain `make -n` misleadingly printed 402 compilation commands.
Generated C++ sources have empty recipes depending on completion stamps touched
slightly after export. A separate minimal GNU make 4.4.1 control demonstrates
that dry-run prints a recompilation in this situation, whereas actual make
correctly leaves an object newer than both prerequisites untouched. This is
retained in `output/probes/make-empty-recipe-control/`; the 402-command dry run
is **not** evidence of 402 real remaining compilations.

For a diagnostic dry run only, passing `-o` for the 95 existing completion
stamps removes that overprediction while preserving source-versus-object and
missing-object checks. Sector 96 is never ignored. The adjusted constituent
dry-run contains nine C++ compilations and one FORM/export action; the outer
wrapper adds one QMC-template compilation and linking. GNU documents the
semantics of `-o` in its [option reference](https://www.gnu.org/software/make/manual/html_node/Options-Summary.html).
The actual continuation keeps **plain native make**, with no `-o`, touched
source timestamps, makefile edit or changed scientific input.

The coordinator approved a third copied-package attempt with **300 seconds for
native make, 180 seconds for numerical integration, 500 seconds overall and the
same 30-GiB process-tree RSS cap**. This reserves work for one remaining FORM
sector, ten C++ translation units and linking; the bound does not assert that
those costs are known. A new native repeated-line projection would require
fresh decomposition/subtraction/code generation, so completing this retained
package is the cheaper justified next attempt. The separate issue-1 and hard
orthant launchers remain prepared but unexecuted.

The new ignored launcher `continue_double_box_reference_attempt3.py` is a copy
of the independently reviewed continuation with only stage bounds and parent
record names changed. It hash-checks the copied second package and preserves
its completion, source/object audit and dry-run records. Native make remains
`make -j1 pylink FORMTHREADS=1 FORMOPT=2`; all numerical settings, seed 20261203,
the Symbolica import guard and full physical tuple capture are unchanged.
Attempt three has started on CPU 0 concurrently with the independently bounded
native point-first oracle on CPU 1. Neither is a performance timing comparison.
The attempt completed successfully and its complete outcome was independently
audited as described below.

## Third attempt: complete independently audited vector

Unchanged native make/link completed in **30.849 seconds**. The numerical call
completed in **47.695 seconds** after a 0.014-second library load. All process
exits were zero and all stage/outer bounds were respected. The copied parent
package remained unchanged. These are diagnostic durations, including explicitly
recorded overlap with the native point-first oracle, not comparative performance
measurements.

Independent HEPKit review verified ten parent records, six guarded sources,
all 2,097 parent package files (77,406,484 bytes), five final source/build hashes,
the absence of Symbolica imports, and exact equality of the tagged tuple with
its separately retained Python representation. The physical tuple is element
two; it equals tuple element zero and the outer prefactor is one. The generated
constituent prefactor is the expected `-Gamma(3+2*eps)` series and the generated
weighted wrapper applies it once. No extra Gamma convolution is permitted.

The returned real coefficients and standard errors are:

| Epsilon order | Value | Native standard error |
| --- | ---: | ---: |
| -4 | -1.0269562977782698e-15 | 2.1150505867918003e-15 |
| -3 | 1.500432835114506 | 0.00036128998422068834 |
| -2 | 1.269034715961728 | 0.0016102860609434047 |
| -1 | 2.997315028586949 | 0.006481159774516808 |
| 0 | -14.883187577966803 | 0.027253699339267728 |

All imaginary values and reported imaginary errors are zero and remain in the
raw tuple. The numerical leading coefficient is not promoted to an exact zero;
the separate analytic proof remains distinct evidence. Native logs show
individual 10,061-point, 32-shift calls, but their aggregate work has not been
audited: actual total evaluations and selected aggregate lattice metadata remain
unknown. The `maxeval=65536` request is not substituted for measured work.
External joint covariance was not returned and is not reconstructed from errors.

The evidence directory is
`output/diagnostics/remaining-pysecdec/double-box-attempt-3/`, including
`result.json`, `raw-series.repr.txt`, `native-integration-input.json`,
`completion.json` and `independent-review.json`. This establishes a usable
independent full-vector correctness observation at the recorded point. It does
not establish convergence, uncertainty calibration or performance parity.

The existing native `ReferenceResult` writer and `reference::compare` adapter
compared all five orders against the retained complete Rust 64-shift vector
(`seed=18932`, 6,684,672 evaluations). Before the reference was marked checked,
the native report retained the expected `UnverifiedReference` status and every
diagnostic pull. After independent source/outcome review, the stored evidence
permits statistical comparison with the explicitly independent external seed
20261203. Signed pulls in order minus four through zero are
`[-0.30750, 0.20811, -0.16910, 0.96726, 1.26516]`. No coefficient selection,
new sampling or alternative error calculation was used. Native joint covariance
is retained in the comparison evidence; missing external covariance remains
missing.

The new fixture is `examples/references/double_box.json`, with source, tuple,
package and audit hashes, the exact kinematic point, explicit real projection
and original uncertainties. It leaves the historical
`examples/targets/double_box.json` and run card unchanged. The pure-data recorder
is `output/probes/record_double_box_reference.rs`; before/after native comparisons
are retained in `output/reference-fixtures/double-box-native-reference/`.
The matrix's calibration requirement remains open despite this full-vector
independent correctness check. The existing fixture transport target passes
both active tests, including complete order/uncertainty/projection preservation
and current native input hashes; its two expensive-data recorder tests remain
ignored.
