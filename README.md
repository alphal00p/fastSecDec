# FastSecDec

Native Rust sector decomposition and numerical integration for Feynman integrals,
built around HEPKit, Linnet, Symbolica, and Numerica/Havana.

The first phase is under active implementation. The complete design, scientific
scope, milestone gates, and original requirements are in
[FIRST_PHASE_PLAN.md](FIRST_PHASE_PLAN.md). The standalone CLI is being developed
alongside the library; the future Python bridge belongs to HEPKit.

With the local dependencies described in the development guide, enter
`nix-shell` and run a native graph or a small analytic direct integral:

```sh
cargo run -- run examples/runs/triangle.toml --points 4096 --shifts 16 --workers 2
cargo run -- run examples/runs/analytic_endpoint.toml --points 4096 --shifts 16
cargo run -- --json inspect examples/runs/double_box.toml --expressions
```

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
a stored selection. Inspect a generated artifact to view its retained chart,
coordinate-map and domain metadata. See the [CLI guide](crates/fastsecdec-cli/README.md)
for scope, checkpoint and reference-export rules.

- [Development environment and dependency setup](docs/DEVELOPMENT.md)
- [Reference regression traceability](docs/REGRESSION_MATRIX.md)
- [Correctness and performance comparison protocol](docs/BENCHMARK_PROTOCOL.md)

The workspace separates the public physics library (`fastsecdec`), exact sector
geometry (`fastsecdec-sectors`), and command-line orchestration (`fastsecdec-cli`).
The QMC library extension lives on a separate branch of Numerica. Reference
checkouts and generated artifacts are deliberately excluded from this repository.
