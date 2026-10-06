# FastSecDec

Native Rust sector decomposition and numerical integration for Feynman integrals,
built around HEPKit, Linnet, Symbolica, and Numerica/Havana.

The first phase is under active implementation. The complete design, scientific
scope, milestone gates, and original requirements are in
[FIRST_PHASE_PLAN.md](FIRST_PHASE_PLAN.md). The standalone CLI is being developed
alongside the library. The optional [Python bindings](bindings/python/README.md)
live in an isolated FastSecDec crate; HEPKit registers their public module through
[draft PR #18](https://github.com/symbolica-dev/symbolica-community/pull/18).
The numerical workspace and default CLI remain independent of Python.

The [marimo showcase](examples/hepkit/README.md) uses native HEPKit inputs,
separate Generate and Integrate actions, live status views and sector inspection.
The scalar triangle is selected by default; the projected `g g -> H H`
double box is an optional longer calculation in native and browser execution.
Its [build guide](examples/hepkit/BUILD.md) documents the community build
without local dependency patches. The updated local native wheel passes
118 controls; the preceding published API milestone passed 95 in hosted native
CI. The earlier `a3d09e` Wasm wheel passes generic
smoke and all 58 collected portable controls. Actual triangle browser execution
covers native metadata/math inspection, QMC checkpoint resume and same-kernel
Havana pilot/production pause and resume. The final supplemental screenshot
failure is retained separately from the accepted numerical lifecycle.
See the [current portable review](docs/reviews/hepkit-metadata-mc-portable.md)
for the bounded optional double-box outcome, exact source identities and
single-thread browser interaction limits. These checks do not establish
browser convergence or representative performance parity.

The [standalone gg→HH notebook](examples/hepkit/gghh.py) builds its diagram from
`Model.standard_model()` with inline masses and helicities, calls
`diagram.sector_decompose(progress="auto")`, and uses ordinary marimo editor
controls for its expensive cells. Generation shares HEPKit's progress presenter;
the final bibliography uses `get_citations()` and offers a BibTeX download.
The current Wasm wheel passes generic smoke and all 89 portable controls,
including the new Model, progress and citation APIs. Native/editor and portable
validation scope is recorded in
the [notebook review](docs/reviews/gghh-single-notebook.md).

Those wheel results precede the latest QMC relocation and upstream dependency
migration. QMC now lives in `crates/fastsecdec-qmc`, alongside ordinary registry
Numerica for Havana MC. The [dependency review](docs/reviews/regular-hepkit-build.md)
records current validation separately from historical wheel results.

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
SymJIT kernels use O2 even in a development build; performance comparisons also
require a release build of the Rust orchestration code.

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
provides an additional representation identity for diagnostics and reuse;
existing artifact, checkpoint and sector-index conventions remain unchanged.
