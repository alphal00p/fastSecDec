# FastSecDec

Native Rust sector decomposition and numerical integration for Feynman integrals,
built around HEPKit, Linnet, Symbolica, and Numerica/Havana.

The first phase is under active implementation. The complete design, scientific
scope, milestone gates, and original requirements are in
[FIRST_PHASE_PLAN.md](FIRST_PHASE_PLAN.md). The standalone CLI is being developed
alongside the library. The optional [Python bindings](bindings/python/README.md)
live in an isolated FastSecDec crate; HEPKit registers their public module through
[merged PR #18](https://github.com/symbolica-dev/symbolica-community/pull/18).
The [eager notebook bridge update](https://github.com/symbolica-dev/symbolica-community/pull/22)
pins the APIs used by the notebooks below.
The numerical workspace and default CLI remain independent of Python.

The [gg→HH marimo notebook](examples/hepkit/gghh.py) starts with native
HEPKit generation of all one- and two-loop diagrams at QED order two, with
Higgs, gluon and top particles and symmetrized initial/final states. Its default
selection is the first one-loop diagram. Separate buttons build the catalogue,
generate the selected integral, inspect sectors, and integrate with QMC or
Havana. The [scalar showcase](examples/hepkit/fastsecdec_showcase.py) uses the
same controls for a smaller triangle example.

Both notebooks use one caller-driven thread and native eager evaluators, including
local runs. Generation and integration retain completed work across pauses;
individual native algebra operations finish before a pause takes effect.
Model inputs and kinematic dot products remain runtime evaluator parameters.
The [notebook guide](examples/hepkit/README.md) and
[build guide](examples/hepkit/BUILD.md) explain native and Pyodide execution.
Historical wheel validation remains recorded with its original source identities
in [the portable review](docs/reviews/hepkit-metadata-mc-portable.md); it does not
certify the current notebook or a newly built wheel.

Cargo fetches the public dependencies directly; no checkout or source-patching
script is required. Enter the development shell and run:

```sh
nix-shell
cargo metadata --format-version 1 --locked
cargo run --locked -- run examples/runs/triangle.toml --points 4096 --shifts 16 --workers 2
cargo run --locked -- run examples/runs/analytic_endpoint.toml --points 4096 --shifts 16
cargo run --locked -- --json inspect examples/runs/double_box.toml --expressions
```

The workspace selects upstream Symbolica's `community` source through Cargo,
and the lockfile records the resolved public dependency identities. See the
[development guide](docs/DEVELOPMENT.md) for portable-library and HEPKit
consumers and the build/runtime validation boundaries.

Interactive terminals show a live dashboard. `--plain` selects text progress;
`--json` emits the final structured report, and `--status-json` streams status
snapshots to stderr. Generated artifacts and checkpoints default to `output/`.
The CLI uses SymJIT O2 by default. `[generation.evaluator] backend = "eager"`
selects native eager evaluation; notebook compilation selects eager explicitly.
Evaluator batches default to 256 points (`--evaluation-batch-size`), independently
of statistical batches and QMC lattices. Performance comparisons also require a
release build of the Rust orchestration code.

Native HEPKit master and reduction comparisons now cover twelve scalar and
eight numerator points, including rank five and a zero Gram determinant. An
additional coupled two-loop numerator agrees with independent parameterization
and analytic Laurent coefficients at two spacelike scales. A
complete 64-shift double-box diagnostic is consistent with its independently
proved zero leading pole. Complete multiloop certification and matched
performance acceptance remain in progress; historical targets with unknown
uncertainties remain explicitly unverified.

Add `--save-result output/result.json` to `run` or `integrate` to save accepted numerical results.
`show-result output/result.json` reads them without the input graph or compiled
artifact. `export-reference output/result.json --source estimate --output
output/reference.json` explicitly selects the computed estimate; `--source
stored` selects the original comparison target. Saving a result does not grant
it independent validation.

For diagnostics, `--sectors 0,3 --exact-contributions include` selects compiled
kernel IDs and retains an explicitly qualified result. `--full-integral` clears
a stored selection. `inspect output/gghh_double_box.fsd` summarizes the saved
generation and lists the ten largest sector evaluators. Add `--sector 5` to see
that kernel's endpoint monomials, coordinate maps and evaluator statistics.
See the [CLI guide](crates/fastsecdec-cli/README.md)
for scope, checkpoint and reference-export rules.
The [gg→HH double-box guide](examples/gghh_double_box/README.md) includes
eight-worker Havana and QMC commands targeting respectively 1% and 0.1% in ε⁰.
The [runtime integration guide](docs/RUNTIME_INTEGRATION.md) explains precision
routing, live estimates and timing diagnostics.

- [Development environment and dependency setup](docs/DEVELOPMENT.md)
- [Reference regression traceability](docs/REGRESSION_MATRIX.md)
- [Correctness and performance comparison protocol](docs/BENCHMARK_PROTOCOL.md)
- [Software and method citations, with BibTeX](citations/README.md)

The HEPKit notebooks end with `symbolica.get_citations()`, which includes
FastSecDec, pySecDec and the geometric sector/subtraction method papers after
native FastSecDec use. Saved kernels also register these citations when loaded.

The workspace separates the public physics library (`fastsecdec`), exact sector
geometry (`fastsecdec-sectors`), caller-driven lattice integration
(`fastsecdec-qmc`), and command-line orchestration (`fastsecdec-cli`).
QMC lives in this repository; Numerica supplies the numeric backends, RNG and
ordinary Havana Monte Carlo. Reference checkouts and generated artifacts are
deliberately excluded from this repository.

Library callers can retain a `GenerationContext` for exact geometry reuse.
The sector crate also exposes [caller-scheduled chart and cone jobs](docs/reviews/parallel-geometry-implementation.md),
with deterministic native merging and cancellation. `KernelSet::sector_content_id`
provides an additional representation identity for diagnostics and reuse without
changing the numerical kernel's sector-index mapping or accepted statistics.
