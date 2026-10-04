# Scientific examples

These inputs use native HEPKit compact DOT. Models and parameter cards use the
existing FeynKit JSON format. Run cards and end-to-end validation are being added
as the generation and CLI slices land; a graph's presence alone is not a claim
of numerical parity. The 19 native graph run cards now in `runs/` cover these
topologies, numerator variants, off-shell kinematics and massive one-loop cases.
End-to-end validation of these cards is pending the CLI implementation.

`models/scalar.json` provides the real mass parameter `mt`, zero widths, and
scalar vertices of degree two through five. `models/massless.json` and
`models/massive.json` set `mt` to zero and one. Native expressions refer to that
parameter as `UFO::mt`. Kinematic invariants remain independent inputs, including
off-shell external legs. The minimal `massless_phi3.json` model also supports the
initial bubble and triangle library examples.

The scalar graph topologies follow the examples in FastSecDecPathFinder revision
`582d8c7f6dde9bf750750d4c2a2d85a94ce940cd`. Particle attributes replace the old
frontend's explicit mass and momentum annotations. HEPKit owns routing; the
external and loop momenta use its existing symbols and basis. Numerator examples
have a separate exact propagator-multiset and numerator test in
`crates/fastsecdec/tests/example_inputs.rs`.

For the box numerator fixtures the historical loop variable is minus the native
momentum on edge 7; the sign is included explicitly in the native numerator.
For the triple-box numerator, native loop edges 6, 8 and 12 directly give the
three historical loop variables.

The historical triangle numerator has an inconsistent unused kinematics
annotation: its explicit propagators `k^2`, `(k+p0)^2`, `(k+p0-p1)^2`, together
with `p0^2=s`, `p1^2=0` and `p0.p1=-s/2`, imply `(p0-p1)^2=2*s`. Its old shared
kinematics file also listed `p2^2=0`, but `p2` was absent from that numerator's
propagator definition. `runs/triangle_numerator.toml` reproduces the integral
actually defined by those propagators with consistent native momenta: the second
outgoing leg has virtuality `2*s`. The scalar triangle card continues to describe
the original one-off-shell, two-on-shell triangle.

| Graph family | Loops | Propagators | Current coverage |
|---|---:|---:|---|
| Bubble | 1 | 2 | Native input, normalization and U/F tests |
| Triangle | 1 | 3 | Native input fixture |
| Box | 1 | 4 | Native topology fixture |
| Double box | 2 | 7 | Native topology fixture |
| Triple box | 3 | 10 | Native topology fixture |
| Kite | 2 | 5 | Native topology fixture |
| Self energy | 3 | 7 | Native topology fixture |
| Three point | 2 | 5 | Native topology fixture |
| Three point, six lines | 2 | 6 | Native topology fixture |
| Three point | 3 | 7 | Native topology fixture |
| Three point, eight lines | 3 | 8 | Native topology fixture |

The hard direct-polynomial expressions in `parametric/four_loop_hard_{u,f}.sym`
retain their original coefficients. Their original domain is the **positive
orthant**, with density `U * F^(eps-3)` and unit prefactor. U is negative and has
integer power one. Do not reinterpret this as a projective simplex or an
original unit cube.

`targets/` extracts the scientific vectors from the three historical JSON
outputs, retaining source revision and origin. Triangle and box contain reported
pySecDec errors. Despite its source filename, the double-box target identifies
itself as manually supplied `numeric` data with zero placeholders for errors;
its independent uncertainty is therefore recorded as unavailable. Those zeros
must never be treated as an exact scientific reference.

Full regression and performance acceptance remains tracked in
[the regression matrix](../docs/REGRESSION_MATRIX.md) and
[the benchmark protocol](../docs/BENCHMARK_PROTOCOL.md).
