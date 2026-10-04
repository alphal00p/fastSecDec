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
before copying. No continuation has been launched. Issue-1 and the other
prepared commands likewise remain queued for coordinated runtime slots.

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
launch. Native FastSecDec production code and its integration settings are
unaffected.
