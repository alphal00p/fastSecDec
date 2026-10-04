# External reference evaluator check

The reference is frozen at `582d8c7f6dde9bf750750d4c2a2d85a94ce940cd`.
Its existing Python program is used only as an external development oracle.
No Python implementation or pySecDec production dependency enters FastSecDec.

The installed environment on Linux x86-64 contains Python 3.12.14,
Symbolica 2.1.0, pySecDec 1.6.6, QMCPy 2.3, NumPy 2.5.0 and SciPy 1.18.0.
The Symbolica wheel's CycloneDX manifest identifies its native Symbolica revision
as `9135b4091505276baa11ef262fe1165f2b95ea33` and embedded SymJIT as **2.18.6**.
The Rust implementation uses its separate pinned Symbolica 3.0.1 worktree and
published SymJIT 2.26.0. These environments are not identical dependencies.

Four runs use the same native reference `dot_box.yaml`, `s=t=-1`, the complete
Laurent vector through order zero, explicit substituted kernels, full sector
support, Kuo/QMCPy linear lattice, Korobov3, 1,024 points, eight shared shifts,
seed one, one worker, and a single integration round. The density includes the
global `Gamma(2+eps)` convention. The result JSONs preserve the exact commands.

| Reference evaluator | eps^-2 | eps^-1 | eps^0 |
|---|---:|---:|---:|
| Analytic / frozen pySecDec target | 4 | -2.308862660 | -12.493116687 |
| Eager real | 3.999999712 | -2.308862061 | -12.493115539 |
| SymJIT O2 complex | 3.999999712 | -2.308862061 | -12.493115539 |
| SymJIT O2 real, indirect translation | 3.999999712 | -1.615893957 | -5.237428123 |
| SymJIT O2 real, direct translation | 3.999999712 | -2.047255311 | -5.425243711 |

The two correct paths agree at the serialized precision. The incorrect real-JIT
results differ by many thousands of their reported standard errors. This
isolates the observed disagreement to the reference real-JIT execution path;
it does not yet identify whether Symbolica's adapter or its embedded SymJIT is
responsible. Reference source and dependencies have not been patched for this.

Reproduce from the reference checkout, substituting one evaluator option group:

```sh
SYMBOLICA_HIDE_BANNER=1 .venv/bin/python FSD.py run \
  --run examples/runs/dot_box.yaml \
  --target examples/outputs/dot_box_pysecdec_target.json \
  --explicit --jit-compile --jit-optimization-level 2 --complex-evaluator \
  --sampling-mode qmc --qmc-lattice-backend qmcpy --qmc-order linear \
  --qmc-support-mode full --qmc-refine-sectors democratic \
  --no-qmc-optimized-evaluators --samples-per-iter 1024 --qmc-shifts 8 \
  --max-iter 1 --min-iter 1 --workers 1 --batch-size 1024 \
  --quiet-summary --no-progress --json --restart \
  --result-path /common/dev/fastsecdec/output/baseline/box-complex-jit-trial-result.json
```

Use `--eager-evaluator --real-evaluator` for eager execution, or replace
`--complex-evaluator` by `--real-evaluator` for the failing real-JIT path;
`--jit-direct-translation` selects the other failing variant. The banner setting
controls display only. The work/sample count and numerical input stay fixed.

Ignored evidence is in `output/baseline/box-{qmc,eager,direct-jit,complex-jit}-trial-result.json`.
Earlier triangle/box ordinary real-JIT timing probes are diagnostic only and
must not become accepted performance baselines. Correct complex O2 timings
still require the repeated, matched protocol and numerical certification.
