# Projected off-shell triple-box reference attempt

The next bounded attempt is the scalar off-shell triple box; the rank-two
numerator follows as a separate fresh attempt. Both use the unchanged native
pySecDec propagator/Gaussian implementation and the independently proved
coefficient-one power projection. No FastSecDec polynomial, subtraction,
coefficient or estimate is supplied to the reference engine.

The initial shared deadline is 600 seconds, with a 30-GiB process-tree RSS bound
and at most 180 seconds for numerical evaluation. These are reference-science
bounds, not timing targets. Each stage receives only its remaining share of the
outer deadline; a timeout preserves the partial package and does not authorize
an automatic continuation. One allowed CPU, one FORM thread, one package
process and one numerical worker are used. Generation needs the exclusive
Symbolica runtime. Guarded native FORM/C++ and numerical phases may support a
separately coordinated continuation later.

## Existing native API and normalization

`LoopIntegralFromPropagators` receives the original ten literal propagators,
loop/external bases and exact replacement rules from the frozen reference DOT
and kinematics readers. The original-order powers are
`[1,1,0,2,0,2,1,1,1,1]`; native zero-power removal must produce the active
`[1,1,2,2,1,1,1,1]`. The off-shell point remains all external virtualities minus
one with `s12=s23=-2`, massless internal lines and `D=4-2*eps`. The scalar numerator
is one; the other attempt uses the literal original rank-two expression
`k1(mu)*k3(mu)+2*k2(nu)*p1(nu)*k2(rho)*p2(rho)`. The graph weight is one.

The installed public `loop_package` API explicitly accepts
`package_generator=pySecDec.code_writer.make_package`
(`loop_integral/loop_package.py:340–365`). This emits an ordinary integral
directly. Its `LoopPackage` adapter owns the raised-power measure, Gaussian
numerator and Gamma factor (`loop_package.py:52–96`). An additional prefactor of
one is explicit; no projected Jacobian, factorial or Gamma convolution is added.
This avoids the one-integral amplitude wrapper and uses the same ordinary
`IntegralLibrary(together=True)` uncertainty path verified for Issue 1.

The ordinary interface sums every sector integrand before native QMC estimates
each Laurent coefficient. Physical tuple member two already owns the generated
prefactor once. The runner saves the full original string tuple before calling
the existing `series_to_json` converter. It retains complex observations and
all returned powers through zero; absent powers are not filled. Before any
fixture is accepted, review the generated prefactor, library metadata, complete
order vector, actual sum branch and native allocation logs. Cross-order
covariance remains unavailable unless the provider explicitly supplies it.

## Prespecified numerical steering

Use the existing published `cbcpt_dn1_100` lattice, Korobov3, no fit function,
N8311/R32, explicit seed20261201 for scalar or20261202 for rank two, one CPU
thread, `epsrel=0.01`, `epsabs=1e-12`, and `maxeval=265952`. This changes the old
unexecuted draft's implicit QMC defaults into an explicit initial allocation.
Requested work is not observed work or an accuracy certificate. If native logs
confirm one allocation per coefficient, 265952 counts scalar **summed-coefficient
point evaluations per coefficient**; each such point evaluates the sector sum.
Actual counts, additional iterations and returned errors must be recorded from
the provider, without reconstructing an estimator.

The draft is `output/probes/run_projected_triple_together.py`. It reuses the
reviewed immutable recording/watchdog helpers and fail-closed Symbolica import
guard from the earlier reference continuation. Its generation subprocess exits
before compilation or numerical loading. Native source/tool/executable hashes,
current scientific input hashes, exact argv and stage outcomes are retained.
The ordinary library/converter import control must pass under the guard before
allowing a guarded phase to overlap other untimed native science.

After source review and explicit runtime handoff, the scalar command is:

```sh
DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder/.venv/bin/python \
  output/probes/run_projected_triple_together.py scalar \
  /common/dev/fastsecdec/output/diagnostics/remaining-pysecdec/triple-scalar-together-attempt-1
```

Rank two uses `rank2` and its own `triple-rank2-together-attempt-1` directory;
that attempt has not run. The independent HEPKit source review finds no scientific or
blocking API issue; the native Gaussian/Gamma owner `loop_integral/common.py`
and native uncertainty header are included in the recorded source hashes.
Standard-library AST parsing and the authorized guarded import controls pass.
The latter are retained under
`output/diagnostics/reference-import-controls-20261005T020026Z`: the negative
control rejected Symbolica before loading, then this frozen runner imported
ordinary IntegralLibrary and its converter under the same guard. Both exits
are zero with empty stderr. Native timer input/executable checks and the final
SHA256 manifest confirm the source remained unchanged. No generation or
integration was performed by these controls. Actual preparation/outcome
evidence remains a separate acceptance step.
The old `run_remaining_reference.py` and
`projected_triple_reference.py` stay unchanged as historical source drafts; new
execution uses this explicitly ordinary-sector-sum route. No production Python,
new algebra or new statistics engine is introduced.

The launch was prepared on CPU0, awaiting the coordinator's explicit handoff
after native representative acceptance and the production commit. The
generation subprocess must exit and be reaped before another Symbolica owner
starts. Only subsequent guarded FORM/C++ and numerical work may overlap
coordinated, untimed native science on another CPU. Existing 600/180-second and
30-GiB bounds remain unchanged.

## Retained scalar attempt one

After production commit `6332676`, the coordinator launched the frozen runner
on CPU0 in `output/diagnostics/remaining-pysecdec/triple-scalar-together-attempt-1`.
Generation passed in 136.321 seconds: eight active parameters with the expected
positive powers, 1182 sectors, and native `gamma(4+3*eps)` expanded through the
required prefactor order three. The generation child exited and was reaped;
an empty process-group check is retained before the explicit handoff to native
FastSecDec work. Subsequent reference compilation ran under the import guard.
Other-CPU release compilation and later native science make these diagnostic
stage durations unsuitable as matched performance measurements.

The original 600-second watchdog expired during compilation. The outer process
reported exit124 after 601.680 seconds including shutdown. All owned process
groups were reaped, the frozen sources are unchanged, and the package retains
136 completed FORM stamps and 412 native object files. Its 4313 files total
40,915,571 bytes; the complete retained manifest has SHA256
`dfd9082ca6cf1055cf8fd30c409a754e3d22a3086e2c539cfb758e837733fdbc`.
The interrupted `src/sector_112_n1.o` is absent, so ordinary make will rebuild
that missing target. No numerical phase, raw tuple, or reference result exists.

This failure remains immutable. The [separately bounded copy continuation](projected-triple-reference-continuation.md)
completed compilation but timed out during its180-second numerical phase, with
no physical tuple. Both failures remain retained. The separately approved
[numeric-only attempt three](projected-triple-reference-numeric-continuation.md)
subsequently returned the complete physical vector; its independent outcome and
native transport evidence are recorded separately from those failures.
