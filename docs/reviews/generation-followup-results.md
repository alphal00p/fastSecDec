# Generation and runtime-model acceptance (2026-10-07)

This record covers the corrected D05 s-channel input, deterministic generation
scheduling, runtime model inputs and the CLI follow-up. It does not migrate
other examples or the existing tests/gates. The original supplied DOT and raw
diagram assets remain unchanged.

## Final model-parameter artifact

The local release CLI generated `output/gghh_double_box.fsd` with eight workers
and SymJIT O2. The artifact contains 30 six-dimensional kernels, complex
coefficients at epsilon orders -1 and 0, and 21 runtime inputs: the fifteen
declared external Gram entries followed by `model::Gf`, `model::MT`, `model::MZ`,
`model::aEWM1`, `model::aS` and `model::ymt`. Each sector evaluator therefore has
27 inputs including its six integration coordinates. The model's native
dependency resolver selected these inputs; the list is not hardcoded in the
frontend or exporter.

The JSON metadata is 14,858 bytes; the context-aware version-six binary is
3,666,802 bytes. Model values in JSON are suggested defaults only. The saved
template requires explicit binding and records the generic nonzero real `MT`
domain. No threshold certification is performed. A recursive metadata scan
found no absolute path values; a moved and renamed JSON/DAT pair cold-loaded
successfully through the CLI.

A second full generation with four workers produced a **byte-identical `.dat`
file**, identical kernel layout and the same semantic evaluator identity
`66f2af45b499f63bb15feb7148d0941eabb654ec2e83227443c023756208f09d`.
Human provenance paths and measured timings depend on destination/run and are
not a claim of byte-identical JSON. Earlier focused scheduling probes also
cover one-worker execution and deliberately reversed completion order.

Observed eight-worker generation took 88.509 s: input 0.210 s,
parameterization 3.884 s, mapping 26.909 s, symmetry 20.197 s, coefficient
expansion 35.814 s and compilation 1.097 s. Status records show eight concurrent
workers and a sampled peak process RSS of 2,843,541,504 bytes. These are local
observations, not a controlled performance comparison with the earlier template
whose model parameters were specialized.

The separate scheduling-only comparison retains that earlier specialized
model: the new parallel scheduler produced byte-identical evaluator data and
reduced the observed symmetry stage from 39.581 s to 8.024 s. Preparation
accounted for 8.001 s and deterministic exact admission for 0.022 s. See the
[independent scheduling review](generation-followup-review.md).

## Full-integral smoke check

The final artifact was cold-loaded and integrated with the example's point file,
1,024 points per shift, four shifts, seed 20261005 and eight workers. All
122,880 evaluations completed, with 16,320 precision rescues, maximum precision
256 bits, and zero failures. Complete Laurent-vector covariance was retained.

| Component | Mean | Standard error |
|---|---:|---:|
| Imaginary epsilon^-1 | -11.54403535047407 | 0.1229682135183347 |
| Imaginary epsilon^0 | 356.3642868168813 | 3.8718653013373503 |

Both real components are zero. Both means exactly match the preceding
specialized-model D05 smoke allocation; the largest covariance entry difference
is 2.49e-14. The run stopped at its work limit and is not a convergence or new
independent amplitude-reference claim. Its 45.040 s integration time was measured
while a separate generation and exporter build were active, so it is not a
performance benchmark. Cold artifact loading took 0.805 s.

An independent direct evaluator probe matched the native chart/representative
maps and compared four interior/endpoint points for each of the 30 sectors,
plus the exact offset: 121 complete vectors. The runtime default point agrees
with the prior specialized template to a maximum relative infinity-norm error
of 1.781e-14. Doubling `ymt` or `aS` separately multiplies every vector by four,
with zero measured discrepancy. Increasing `MT` by ten percent changes the
sampled outputs. Attempting `MT=0` rejects the rebind before changing the prior
valid point identity. Binding leaves the template identity and binary bytes
unchanged.

## Checks and ownership

- Release CLI build, strict scoped core/CLI Clippy, formatting and Python binding
  `python_stubgen` checks pass with the installed Rust 1.99 toolchain. This host
  has no `nix-shell`.
- Invalid `.json` generation output was rejected in 0.017 s even with a missing
  input card, before input loading or dashboard initialization.
- Independent native and portable host probes cover dependency resolution,
  complex input phases, default-zero named masses, mass-domain checks, failed
  rebinding, exact offsets, worker cloning, weighted precision replay, cold
  Symbolica-state remapping and version-five artifact compatibility. Actual
  Wasm/Pyodide execution remains a separate later task.
- Native DOT presentation preserves the complete imported diagram payload.
  HEPKit rejects omission of the `v0` numerator with `NumeratorFragmentMismatch`;
  that required duplicate remains. Every graph attribute is on its own line.
- A fresh release exporter reproduced all fourteen scientific/source assets
  byte for byte, including the runtime point file and formatted DOT. Provenance
  differs only in the measured exporter duration.
- Existing tests and other examples are untouched. The deferred old symmetry
  tests still have incomplete test-only runtime-input initializers; no full
  workspace test-gate success is claimed.

The independent [native-model review](runtime-model-parameters-review.md),
[runtime and DOT review](generation-dot-and-runtime-review.md),
[parallel-symmetry record](parallel-symmetry.md),
[memory review](generation-memory.md) and [preflight review](generation-preflight.md)
record API reuse, concrete probes and remaining boundaries. Raw logs, temporary
probe sources and generated artifacts stay under ignored `output/`.
