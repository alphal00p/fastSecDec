# Physical multiloop controls from the LTD paper

These are native HEPKit graph inputs for the required multiloop cases in
[arXiv:1912.09291v2](https://arxiv.org/html/1912.09291v2). They exercise parameter
space contour deformation; they do not reproduce the paper's LTD algorithm.

| Directory | Ancillary record | Loops | Propagators | Reference |
| --- | --- | ---: | ---: | --- |
| `2l4p_k1` | `2L4P.b.K1` | 2 | 7 | Analytic ladder |
| `2l4p_k1_massive` | `2L4P.b.K1*` | 2 | 7 | Table 6, pySecDec |
| `2l6p_a_i` | `2L6P.a.I` | 2 | 9 | Table 2, two numerical references |
| `3l4p_k1` | `3L4P.K1` | 3 | 10 | Analytic ladder; prepare before execution |

The Rust importer uses `FeynmanDiagram`, its native loop basis, `Kinematics`,
`FourMomentum`, `IntegralFamily`, native Symanzik construction and Linnet. The
small topology DOTs specify graph connectivity; they implement no custom graph
parser or momentum router. Every archived denominator is checked against the
native graph before a run card is written.

From the repository root, with the documented native build environment and
current dependency fixes applied:

```sh
nix-shell --run 'cargo test -p fastsecdec --example ltd_contour -- --test-threads=1'
nix-shell --run 'cargo run -p fastsecdec --example ltd_contour -- --output examples/contour/ltd'
./target/debug/fastsecdec inspect examples/contour/ltd/2l4p_k1/run.toml
```

The importer only validates and exports; it never starts sector generation or
integration. `--case '2L4P.b.K1*'` selects one record. Generated cards opt into
fixed contour capability and retain default causal checks. Their initial
strength is an explicit starting setting, not a validated convergence optimum.
Generation and integration should be started separately after the smaller
analytic contour gates, for example:

```sh
./target/debug/fastsecdec generate examples/contour/ltd/2l4p_k1/run.toml --output /tmp/ltd-k1.fsd --serial --workers 1
./target/debug/fastsecdec integrate /tmp/ltd-k1.fsd --contour fixed --lambda 0.001 --workers 8
```

Use a binary built from the contour branch. Do not start the three-loop case as
an initial validation gate. The four-loop `4L4P.a.I` ancillary definition has an
unresolved mismatch with the published threshold case and is deliberately
excluded.

`ancillary-records.json` retains only the four selected numeric records and
their raw YAML block hashes. `validation.json` beside each exported card records
the immutable archive/member hashes, exact external vectors, tiny conservation
reconciliation, denominator signs, native U/F, and references with their stated
uncertainties. Binary floating source values are converted exactly with native
Rational conversion; the dependent external momentum is fixed by conservation.
The common massive value is exactly `2/5`, corresponding to the paper's `0.4`.

The native measure multiplier is
`(i*(4*pi)^eps/(16*pi^2))^L`, so the reported finite coefficient is already in
the paper's `(2*pi)^(-D)` measure. It acts on the full complex Laurent vector;
no real/imaginary component is compared before conversion. The importer test
checks phase and scale through an actually generated one-loop coefficient and
HEPKit's native OneLOop provider. Missing published error estimates remain
unspecified. The massive ancillary zero is a placeholder, not a reference.

See [the provenance and reuse review](../../../docs/reviews/contour-ltd-fixtures.md)
for source pins, topology naming and reference limitations. Import validation is
not a claim of multiloop numerical convergence.
