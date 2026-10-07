# Scientific examples

These inputs use native HEPKit compact DOT. Models and parameter cards use the
existing FeynKit JSON format. End-to-end validation is being extended
as the generation and CLI slices land; a graph's presence alone is not a claim
of numerical parity. The 20 native graph run cards now in `runs/` cover these
topologies, numerator variants, off-shell kinematics and massive one-loop cases.
The inventory contains 24 graph and direct-parametric run cards and 17 native
DOT files; the CLI input test covers every card.
The separate [generated gg→HH double box](gghh_double_box/README.md) adds a
Standard-Model diagram, numerical helicity projectors and its full provenance.
It uses runtime kinematic/model inputs, paired metadata/native-data artifacts,
and the same batched QMC and Havana runtime as the scalar examples. See its guide
for the current commands and the recorded validation scope.
Native bubble and triangle CLI runs pass initial analytic checks, and the native
massless box passes its complete Laurent-vector QMC regression. Eleven native
one-loop graph cases also pass complete-vector comparisons against HEPKit's Rust
scalar masters, including masses, off-shell legs and nontrivial scales. The coupled
sunset numerator additionally passes independent density and complete Laurent-vector
checks. Remaining multiloop and numerator acceptance is tracked in the regression matrix.

`models/scalar.json` provides the real mass parameter `mt`, zero widths, and
scalar vertices of degree two through five. `models/massless.json` and
`models/massive.json` set `mt` to zero and one. Native expressions refer to that
parameter as `UFO::mt`. Kinematic dot products are named runtime inputs, including
nonzero off-shell virtualities. `[integration.parameters]` selects each card's
historical Euclidean point, and contributing nonzero model masses are supplied
there as `model::mt`. Literal zero masses and on-shell virtualities define the generated
family; changing those structural restrictions requires regeneration. The
`[parameters]` compile-time substitution option remains available for deliberate
specialization, but these graph cards use evaluator parameters instead.

```sh
./target/release/fastsecdec generate examples/runs/triangle.toml --output output/triangle.fsd
./target/release/fastsecdec inspect output/triangle.fsd
./target/release/fastsecdec integrate output/triangle.fsd --method qmc --points 4096 --shifts 32 --target-order 0 --relative-tolerance 0.001 --max-rounds 8
```

Always pass the artifact basename. Generation writes human metadata to
`output/triangle.fsd.json` and the native serialized evaluator to
`output/triangle.fsd.dat`. Keep both together when moving the artifact.
`--parameters point.toml` overrides the stored integration point without
recompiling expressions. The [Marimo notebooks](hepkit/README.md) expose the
same separate actions using single-core eager evaluation. The minimal `massless_phi3.json` model also supports the
initial bubble and triangle library examples.

The scalar graph topologies follow the examples in FastSecDecPathFinder revision
`582d8c7f6dde9bf750750d4c2a2d85a94ce940cd`. Particle attributes replace the old
frontend's explicit mass and momentum annotations. HEPKit owns routing; the
external and loop momenta use its existing symbols and basis. Numerator examples
have a separate exact propagator-multiset and numerator test in
`crates/fastsecdec/tests/example_inputs.rs`.

The additional `sunset_2loop_numerator` comes from the reference's inline
two-loop regression rather than its original fifteen DOT files. Its native
loop edges 2 and 3 carry `K(0)` and `K(1)`; the third denominator is
`(K(0)+K(1)+P(0))^2`, and the vertex numerator is
`K(0).K(1)+2*K(0).P(0)`. The card has `P(0)^2=-1` and requests the complete
Laurent vector through epsilon one. Exact routing, an external pySecDec
pointwise Gaussian density, and integrated coefficients at two spacelike
scales are checked in `crates/fastsecdec/tests/sunset_numerator.rs`.
Its analytic identity and measure-sign control are documented in
[the coupled-sunset review](../docs/reviews/coupled-sunset-numerator.md).

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
| Bubble | 1 | 2 | Native input, normalization, U/F and native B0 comparisons |
| Triangle | 1 | 3 | Native input, analytic Laurent vector, CLI integration and native C0 comparisons |
| Box | 1 | 4 | Native input, complete Laurent vector against an analytic identity, frozen external target and native D0 comparisons |
| Double box | 2 | 7 | Native input and exact U/F/measure equality to the independent polynomial fixture |
| Sunset with coupled numerator | 2 | 3 | Native routing, independent Gaussian density, full Laurent vector at two scales and a convergent scalar sign control |
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

`runs/analytic_endpoint.toml` is a small end-to-end check of endpoint subtraction,
the Gamma prefactor, and the complete Laurent vector. Its density is
`eps*Gamma(eps)*x^(eps-1)/(1+x)^2` on the unit interval. Writing
`a = -log(2)-1/2` and Euler's constant as `gamma_E`, the coefficients of
`eps^-1`, `eps^0`, and `eps^1` are respectively
`1`, `a-gamma_E`, and
`pi^2/6+log(2)-gamma_E*a+gamma_E^2/2`.
The independent generation test derives these coefficients from an analytic
integral and checks the complete generated/JIT/QMC pipeline.

`targets/` extracts the scientific vectors from the three historical JSON
outputs, retaining source revision and origin. Triangle and box contain reported
pySecDec errors. Despite its source filename, the double-box target identifies
itself as manually supplied `numeric` data with zero placeholders for errors;
its independent uncertainty is therefore recorded as unavailable. Those zeros
must never be treated as an exact scientific reference.

`targets/issue_1.json` also retains the manual target from the historical issue
card, with unavailable uncertainty. `targets/four_loop_hard.json` transcribes the
full-sector Laurent sum and reported errors from the historical hard-polynomial
report. Its decimal values are rounded, and its uncertainty has not been
independently certified. The report used boundary support grouping; its historical
timings are not a matched full-support performance benchmark for this project.

`references/four_loop_hard.json` now supplies a separately checked full-orthant
reference through order zero. It retains all four external coefficients and
reported errors, including order minus three. The older native numerical vector
starts at minus two and remains unchanged; a separate complete native generation
proves zero through minus three. Its finite reference uncertainty is about
0.57%, above the one-per-mille target. See the
[reference notes](references/README.md) for the comparison and provenance rules.

Full regression and performance acceptance remains tracked in
[the regression matrix](../docs/REGRESSION_MATRIX.md) and
[the benchmark protocol](../docs/BENCHMARK_PROTOCOL.md).
