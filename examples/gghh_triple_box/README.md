# Three-loop contribution with two top-quark loops

This supplied native **D020** graph contributes to `g g -> H H`. It has eight
vertices, ten internal propagators and three independent loop momenta. Its
eight top propagators form **two separate four-edge fermion loops**, connected
by two gluon propagators. Both incoming gluons attach to the first top loop;
both outgoing Higgs legs attach to the second. The two exchanged gluons are in
a **color-singlet channel**: the top loop attached to the two Higgs bosons supplies
`tr(T^a T^b) = T_F delta_ab = delta_ab/2`. This is two-gluon exchange. The
separate projector on the incoming gluons remains the unnormalized color delta
already stored in the graph; no extra color factor or average is introduced.

The top cycles have edges `[4,5,9,11]` and `[6,7,8,12]`; the connecting gluons
are edges `10` and `13`. Native loop routing uses edges `[4,7,10]`, with
independent external coordinates `P(0),P(1),P(2)` and
`P(3)=P(0)+P(1)-P(2)`. This is distinct from
[`gghh_triple_box_bis`](../gghh_triple_box_bis/README.md), whose top edges form
one eight-edge outer loop with two gluon rungs. Neither input is a complete
Standard-Model amplitude or a cross section.

The original enumeration recipe and generation report for D020 are unavailable.
`graph.dot` supplies the complete three-loop graph and its native factors.
No new diagram enumeration is claimed.

## Generation and runtime inputs

The run card selects `generation.mode = "numerical_dual"` and
`generation.contraction_mode = "dots"`. The CLI spelling `dots` selects Idenso's
native dots contraction policy, followed by the input owner's `to_dots`
normalization. This is the requested fully contracted scalar-product route;
the `_bis` example instead requests `minimal`. Deferred sector maps and native
endpoint jets are constructed after numerator preparation. The default local
evaluator remains SymJIT O2; integration is a separate explicit action.

Incoming gluons are exactly massless during generation:
`P(0)^2 = P(1)^2 = 0` is declared with `value = "0"` in `[kinematics]`, before
parameterization and sector finding. The other thirteen Gram products remain
runtime symbols, including polarization products that vanish at the default
point. Model masses and couplings remain analytic runtime inputs. The point
card contains the thirteen Gram values and six model leaves (`model::Gf`,
`model::MT`, `model::MZ`, `model::aEWM1`, `model::aS`, `model::ymt`). Every
`point.toml` value is a decimal float preserving the original runtime f64 input.
The native model resolves dependent couplings; its numeric card provides defaults
and zero-width restrictions without freezing those inputs into the integrand.

The default point uses `sqrt(s)=300`, `mH=125`, `MT=ymt=172.5`, zero widths and
`cos(theta)=4/5`, with incoming `(+,+)` polarizations. Internal algebra retains
`D=4-2*eps`; external wavefunctions are four-dimensional. The loop measure is
`prod_l d^D k_l/(i*pi^(D/2))`, without an extra scale or Euler-gamma factor.
Runtime propagator masses must be finite, real and nonzero. Threshold safety
remains the caller's responsibility; no threshold regularization or
certification is performed.

Regenerate any artifact built with generic runtime `p0p0` and `p1p1`: binding
these inputs to zero only at integration time cannot restore sectors or poles
missing from generic generation. The new point card omits these fixed keys and
is intended for freshly generated on-shell artifacts.

From the repository root:

```sh
./target/release/fastsecdec generate examples/gghh_triple_box/run.toml \
  --output output/gghh_triple_box.fsd --workers 8

./target/release/fastsecdec inspect output/gghh_triple_box.fsd
./target/release/fastsecdec inspect output/gghh_triple_box.fsd --sector 0
```

For Havana sampling, request 1% relative uncertainty on the finite coefficient:

```sh
./target/release/fastsecdec integrate output/gghh_triple_box.fsd \
  --full-integral --parameters examples/gghh_triple_box/point.toml \
  --method discrete_mc --workers 8 --points 32768 --shifts 32 --seed 20261008 \
  --target-order 0 --relative-tolerance 0.01 --absolute-tolerance 0 --max-rounds 8 \
  --checkpoint output/gghh_triple_box.mc.checkpoint.json \
  --save-result output/gghh_triple_box.mc.result.json
```

For randomized lattice QMC, request 0.1% relative uncertainty:

```sh
./target/release/fastsecdec integrate output/gghh_triple_box.fsd \
  --full-integral --parameters examples/gghh_triple_box/point.toml \
  --method qmc --workers 8 --points 4096 --shifts 32 --seed 20261008 \
  --target-order 0 --relative-tolerance 0.001 --absolute-tolerance 0 --max-rounds 8 \
  --checkpoint output/gghh_triple_box.qmc.checkpoint.json \
  --save-result output/gghh_triple_box.qmc.result.json
```

Havana `points` and `shifts` mean global points per batch and independent
batches; QMC uses points per shifted lattice and independent random shifts.
The work limit does not guarantee either target. All Laurent components and
their covariance are retained. See the [runtime guide](../../docs/RUNTIME_INTEGRATION.md)
for refinement, diagnostics and checkpoint semantics.

The base path `output/gghh_triple_box.fsd` identifies adjacent `.fsd.json` human
metadata and `.fsd.dat` native evaluator/expression files. Move both together;
the base itself is not a file. Every resource path here is relative.

## Preserved native factors and validation

The supplied graph already contains a projector with two incoming Lorentz
polarizations and an **unnormalized external color delta**. These remain
unchanged. No color average, spin average or additional diagram symmetry factor
is inserted. Color has not been precontracted for this export: its native
color tensors and delta are processed by the configured numerator contraction.
This differs from `_bis`, whose color delta has already been consumed once by
native color reduction.

The directory contains `README.md`, `run.toml`, `graph.dot`, `model.json`,
`parameters.json` and `point.toml`; no separate numerator snapshots are needed.
In `graph.dot`, the complete tensor numerator is stored once in the native
graph-level `numerator_prefactor`; the global and
local numerators are `1`. The original prefactor is `1`, and the native
bookkeeping owner evaluates the supplied overall factor to `+1`. Native Atom
equality verifies the complete weighted product before and after this storage
change. All momentum routing is unchanged. No tensor contraction is involved
in moving the existing expression between these multiplicative slots.

Strict native graph import, diagram/routing validation, model/card identity,
serialization round trips and the two top-only cycles have been checked.
The model and card agree with the included runtime point defaults. FastSecDec
parameterization, three-loop sector generation, evaluator compilation and
integration have **not** been run. No sector count, pole order, performance
forecast or numerical result is claimed for this input.
