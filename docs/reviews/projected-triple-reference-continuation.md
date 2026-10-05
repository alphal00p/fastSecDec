# Projected scalar copied-package continuation proposal

Scalar attempt one completed native generation but reached its original
600-second deadline during guarded FORM/C++ work. Its 1182-sector package is
retained unchanged with 136 complete FORM stamps, 412 objects, and a full
4313-file manifest. Numerical integration did not start. The interrupted
`src/sector_112_n1.o` is absent; no object or completed-source timestamp will be
altered to bypass native dependency checks.

Propose a fresh `triple-scalar-together-attempt-2` directory and a verified copy
of the entire retained package, preserving modification times. There are 1046
remaining FORM jobs plus pending C++ objects, library templates and linking.
The first bounded compile performed substantial actual work, so finishing this
copy avoids another symbolic generation. Its cost is not known in advance.

The ignored caller `output/probes/continue_projected_triple_together.py` uses
the existing immutable-copy manifest and watchdog helpers. It requires the
retained timeout, successful generation, reaped process groups, no prior
numerical attempt, and exact current hashes for every parent scientific/tool
source. It verifies the retained package manifest, then checks every copied
byte and nanosecond modification time before make starts. Parent contents,
timestamps, record hashes and all guarded sources are checked again afterward.

Use **1200 seconds for plain native compilation**, **180 seconds for numerical
work**, and **1400 seconds total**, with the existing **30-GiB process-tree RSS
watchdog**. Compilation uses eight checked distinct physical cores 0–7,
`make -j8 projected_triple_scalar_pylink.so`, FORMTHREADS1 and FORMOPT2.
The generated Makefile gives each `sectorN.done` one FORM/export recipe and
declares the source/object/library prerequisites. Existing native make owns
parallel scheduling and missing-target rebuilding. No `-o`, touching, source
editing, altered FORM optimization or new dependency logic is used.

All Python FORM/export helpers run under the fail-closed Symbolica import
guard. Both the negative guard and ordinary-library import controls repeat
inside the total deadline before compile. The numerical phase runs on CPU0
through the **unchanged frozen scalar runner**, bound to SHA256
`c14bb4faef75a026d6d09ba83a8d836dd1aea3920e6140043552c6beabb63beb`.
It retains together=True, seed20261201, Korobov3/no fit, N8311/R32,
maxeval265952, one numerical worker, and every physical tuple component. No
generation or Symbolica instance is invoked by this continuation. The outer
numerical limit includes library loading and setup. Subsequent source review
established that ordinary `pylink_integral.hpp:110–127` explicitly ignores the
amplitude `wall_clock_limit` argument; the external watchdog alone enforces this
180-second time bound. Native QMC constructor `maxeval` remains effective.
Full prefactor and
uncertainty-path review is still required before accepting any result.

This is an independent scientific continuation, not a cold-generation or
parallel speedup measurement. Guarded work can overlap coordinated untimed
FastSecDec science on CPU8; no such overlap is assumed without handoff.

After independent source review and explicit coordinator approval:

```sh
/common/dev/fastsecdec/DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder/.venv/bin/python \
  /common/dev/fastsecdec/output/probes/continue_projected_triple_together.py \
  /common/dev/fastsecdec/output/diagnostics/remaining-pysecdec/triple-scalar-together-attempt-1 \
  /common/dev/fastsecdec/output/diagnostics/remaining-pysecdec/triple-scalar-together-attempt-2
```

The runner's AST syntax passes without importing or executing it. Its SHA256
is `42329c44c858a48761089c08cd7b4409bb512c82aeb349a1d88e22c10200d214`.
Independent HEPKit concrete-source review passed: retained-parent admission,
source/content/timestamp checks, native make prerequisites and parallelism,
unchanged complete numerical caller, bounded guarded phases and postflight
checks have no open source finding. See
[the independent audit](projected-triple-reference-independent.md).

## Retained attempt two outcome

The coordinator authorized this exact source, and the fresh continuation ran in
`output/diagnostics/remaining-pysecdec/triple-scalar-together-attempt-2`.
Guard controls passed. Plain native compilation completed all 1182 sector jobs
and linked the ordinary library successfully in **324.766 seconds**, exit zero.
The numerical stage reached its unchanged external 180-second deadline and
returned exit124 after **181.162 seconds** including shutdown. The overall
continuation returned124 after **509.079 seconds**, below its 1400-second bound.
All owned children/process groups were reaped. Parent contents and timestamps,
parent records, and every scientific/tool source remained unchanged.

The final compiled package contains 16,845 files and 235,125,059 bytes. Its
manifest SHA256 is
`2b19194a1c23ac78f7c0c9fe74b2beef6dca0e0e5e56d62ee51941554ed96f47`;
the library SHA256 is
`299ea47a01e76bf71f0562408cfec4d64261ede3b03e85032ed9499e49aff0d2`.
Neither a physical tuple nor a converted result was returned. Partial native
coefficient logs are retained and are **not a reference result**.

The provider first logs a QMC call at113.466 seconds, completes three calls by
137.341 seconds and starts the fourth at137.436 seconds. Its ordinary library
metadata requests the four integral orders−3 through0. The preceding native
messages generate and sum integrands; the logs do not isolate those operations'
individual costs. Each completed call reports8311×32=265952 scalar summed-sector
coefficient point evaluations. No total completed physical work or precision
claim is inferred from the unfinished fourth call.

A [fresh numeric-only proposal](projected-triple-reference-numeric-continuation.md)
uses the verified compiled package and preserves both earlier failures. It is
separately reviewed and bounded; this attempt has not been extended.
