# D05 double box at 1000 GeV

This is the same native D05 s-channel `gg -> HH` double box as the
[400 GeV input](../gghh_double_box_400/README.md), now at `sqrt(s)=1000 GeV`,
`cos(theta)=4/5`, `mH=125 GeV`, and `mt=ymt=172.5 GeV`, with incoming `++`
gluons. It references the unchanged [graph](../../gghh_double_box/graph.dot),
[model](../../gghh_double_box/model.json) and
[model card](../../gghh_double_box/parameters.json).

The existing native `Point::with_sqrt_s` creates the exact external momenta
and HEPKit helicity wavefunctions. Native dot products verify both incoming
mass shells, outgoing Higgs mass shells, momentum conservation and
`s=1000000 GeV^2`. All 15 Gram entries are checked: the two incoming self-products
remain exact zeros before sector discovery; the other 13 are regenerated
runtime inputs. The six model inputs are unchanged. The actual native
`eps1eps2=-0.9999999999999998` normalization is retained.

The parsed generation card is identical to the 400 GeV card. Energy enters
only through `point.toml`; it does not change the graph, projected numerator,
model, regulator or parametric generation input. `point-exact.json` retains
the native exact data and `validation.json` records source identities and
input checks. The 400 GeV momentum-reference check is explicitly inapplicable
at this energy; it is not claimed as a 1000 GeV numerical reference.

The observable is the original individual D05 contribution with unnormalized
`delta_ab` contraction, `D=4-2*eps`, measure
`prod_l d^D k_l/(i*pi^(D/2))`, and multiplier 1. There is no spin/color average,
diagram sum, additional loop prefactor, or individual-diagram Ward claim.
The complete projected tensor numerator and overall factor stay in the
original native DOT. They must not be applied again.

Reproduce the input checks and cards from the repository root:

```sh
nix-shell --run 'cargo run --locked -p fastsecdec --example gghh_double_box_400 -- . target/gghh-double-box-1000-input-check --sqrt-s 1000'
nix-shell --run 'cargo test --locked -p fastsecdec --example gghh_double_box_400 -- --test-threads=1'
```

The exporter requires a fresh output directory. Its default remains 400 GeV;
the tests reproduce both maintained fixtures. Emitted relative asset paths
target this maintained directory, so temporary output cards are for comparison
and must not be executed at arbitrary directory depth.

Input admission has passed. Generation, checked contour admission and numerical
integration are separate actions. This fixture alone establishes no accepted
strength/cap, convergence, variance gain or independent double-box reference.
Saved families may contain undeformed capability, but physical integration at
this point must explicitly select an admitted fixed or dynamic prescription.

The maintained benchmark cards reproduce the corrected fixed-contour campaign:
symbolic endpoint IBP and Laurent expansion, a symbolic contour Jacobian,
SymJIT O2, and initial relative series width 1. The two runtime cards retain
the full Laurent vector, Pilot16 admission, 50 caller-owned worker threads,
and the independently selected fixed strength `lambda=1e-6`. Generation and
integration remain separate actions.

These historical generation cards explicitly request `horner_iterations = 0`
to reproduce the published programs. New generation defaults to **10**. Remove
that override, or set it to 10, for a new production generation; the saved
historical kernels used by the cap study are not retroactively reoptimized.

From the repository root, with the native `fastsecdec` executable on `PATH`:

```sh
fastsecdec generate examples/contour/gghh_double_box_1000/benchmark-generation.toml \
  --recipe fixed-v1 --contour-jacobian symbolic --serial --workers 6 \
  --output target/gghh-double-box-1000-fixed.fsd

fastsecdec --plain --json --status-json integrate target/gghh-double-box-1000-fixed.fsd \
  --parameters examples/contour/gghh_double_box_1000/point.toml --full-integral \
  --integration-settings examples/contour/gghh_double_box_1000/benchmark-qmc.toml \
  --seed 202610102001 --checkpoint target/gghh-double-box-1000-qmc.checkpoint.json \
  --save-result target/gghh-double-box-1000-qmc.result.json

fastsecdec --plain --json --status-json integrate target/gghh-double-box-1000-fixed.fsd \
  --parameters examples/contour/gghh_double_box_1000/point.toml --full-integral \
  --integration-settings examples/contour/gghh_double_box_1000/benchmark-discrete-mc.toml \
  --seed 202610102003 --checkpoint target/gghh-double-box-1000-mc.checkpoint.json \
  --save-result target/gghh-double-box-1000-mc.result.json
```

The cards specify allocations, not a wall timer. The measured runs sent `SIGINT`
to the native coordinator when its integration `elapsed_seconds` reached 300,
then allowed up to 120 seconds for outstanding work, checkpoint and result
writing. This clock includes worker/context setup and Havana adaptation;
artifact loading and the initial causal pilot are reported separately. A
caller can reproduce that boundary using `--status-json`; using shell `timeout`
from process start would include loading and could kill the final flush.
Select 50 distinct physical cores with the host's affinity tooling if matching
the campaign's CPU allocation. The shown seeds reproduce existing runs; use
fresh seeds for independent evidence.

The polynomial campaign uses the same **symbolic endpoint** route with
`--contour-jacobian dual` and initial relative width 2. That choice dualizes
contour-image derivatives only; it does not select numerical-dual endpoint
reduction. The full-artifact selection on separate seeds chose `S=0.8`,
`L=1e-6`,
`R=1`; the polynomial runtime cards preserve those settings. Width changes the first native series attempt, while the same native
remainder-coverage test still determines acceptance. The original `run.toml`
and exporter-produced identities are unchanged.

Reproduce the polynomial construction and its independently seeded runtime rows:

```sh
fastsecdec generate examples/contour/gghh_double_box_1000/benchmark-generation-polynomial.toml \
  --serial --workers 2 --output target/gghh-double-box-1000-polynomial.fsd

fastsecdec --plain --json --status-json integrate target/gghh-double-box-1000-polynomial.fsd \
  --parameters examples/contour/gghh_double_box_1000/point.toml --full-integral \
  --integration-settings examples/contour/gghh_double_box_1000/benchmark-polynomial-qmc.toml \
  --seed 202610102002 --checkpoint target/gghh-double-box-1000-polynomial-qmc.checkpoint.json \
  --save-result target/gghh-double-box-1000-polynomial-qmc.result.json

fastsecdec --plain --json --status-json integrate target/gghh-double-box-1000-polynomial.fsd \
  --parameters examples/contour/gghh_double_box_1000/point.toml --full-integral \
  --integration-settings examples/contour/gghh_double_box_1000/benchmark-polynomial-discrete-mc.toml \
  --seed 202610102004 --checkpoint target/gghh-double-box-1000-polynomial-mc.checkpoint.json \
  --save-result target/gghh-double-box-1000-polynomial-mc.result.json
```

Apply the same native 300-second integration-clock stop and allowed final flush
described above. Generation's initial width and Jacobian policy are independent
of the symbolic endpoint mode. The fixed and polynomial cards both retain the
complete Laurent vector and all covariance entries.

The measured polynomial generation began with two workers and resumed with four
after two durable units. The fixed two-worker command above reproduces the
construction and physics, rather than that historical scheduling sequence.
