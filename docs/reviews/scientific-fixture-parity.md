# Scientific fixture parity audit

This read-only audit compares the migrated native HEPKit inputs with
FastSecDecPathFinder revision
`582d8c7f6dde9bf750750d4c2a2d85a94ce940cd`. It distinguishes the integral being
computed from integration settings and historical numerical targets. It does
not claim completed end-to-end parity or performance acceptance for these
multiloop and numerator cases.

All requested scientific fixtures are present. The CLI process test loads all
23 shipped run cards; the native graph tests separately verify topology,
export/reload, and exact propagator/numerator equivalence for the five numerator
fixtures. Loading and exact input equivalence do not establish agreement of
the integrated Laurent coefficients.

## Measure and Laurent conventions

All graph cards use `D = 4 - 2*eps`, unit propagator powers, the native normalized
per-loop measure `d^D k / (i*pi^(D/2))`, an explicit measure multiplier of one,
and requested maximum epsilon order zero. They introduce no implicit scale or
Euler/Gamma normalization factor.

The matching reference graph convention is `prefactor-convention: pysecdec`.
`src/pysecdec_bridge.py` retains `li.Gamma_factor`, and
`src/formatting.py` convolves it into the displayed coefficients. For a scalar
graph with integer powers this factor is
`(-1)^N * Gamma(N-L*D/2) / product(Gamma(nu_i))`.
The underlying source is `pySecDec/loop_integral/common.py::Gamma_factor`.
Reference graph convention `sector` omits that factor and therefore does not
give the same integral. Tensor numerators also redistribute Gamma factors
between their reduced numerator and global factor; their complete product,
rather than the scalar global factor alone, must be compared.

The direct orthant cards instead specify their complete scalar density and
prefactor explicitly. Their reference convention `sector` and prefactor one
are appropriate. Descriptive reference `loop-count` and `dimension` fields must
not be used to add a topology-derived prefactor to those densities.

Reference display ranges often start at `-2*L` even when leading coefficients
vanish. Those padded labels do not prove that the integral has poles of those
orders. Compare coefficients by their Laurent order, filling absent coefficients
with zero only where the native result proves their absence.

## Graph cases

Here `s` and `t` denote the reference `s12` and `s23`. All masses below are
internal masses; the scalar model parameter `mt` is a mass, not a squared mass.

| Native card | Matching reference input | Scientific point | Scalar prefactor |
| --- | --- | --- | --- |
| `triple_box.toml` | `runs/dot_triple_box.yaml` | Three loops, ten propagators; masses zero; all external virtualities zero; `s=t=-1` | `Gamma(4+3*eps)` |
| `triple_box_offshell.toml` | `runs/dot_triple_box_offshell.yaml` | Same topology; masses zero; all four external virtualities `-1`; `s=t=-2` | `Gamma(4+3*eps)` |
| `triple_box_offshell_rank2_numerator.toml` | `runs/dot_triple_box_offshell_rank2_numerator.yaml` | Same off-shell point; numerator `k1.k3 + 2*(k2.p1)*(k2.p2)` | Full Gaussian numerator and prefactor |
| `kite_2loop.toml` | Graph and corresponding kinematics YAML | Two loops, five propagators; every mass one; external virtuality `-1` | `-Gamma(1+2*eps)` |
| `self_energy_3loop.toml` | Graph and corresponding kinematics YAML | Three loops, seven propagators; every mass one; external virtuality `-1` | `-Gamma(1+3*eps)` |
| `three_point_2loop.toml` | Graph and corresponding kinematics YAML | Two loops, five propagators; every mass one; all three external virtualities `-1` | `-Gamma(1+2*eps)` |
| `three_point_2loop_6line.toml` | Graph and corresponding kinematics YAML | Two loops, six propagators; same masses and virtualities | `Gamma(2+2*eps)` |
| `three_point_3loop.toml` | Graph and corresponding kinematics YAML | Three loops, seven propagators; same masses and virtualities | `-Gamma(1+3*eps)` |
| `three_point_3loop_8line.toml` | Graph and corresponding kinematics YAML | Three loops, eight propagators; same masses and virtualities | `Gamma(2+3*eps)` |

The native off-shell triple-box basis has
`P0.P1=P1.P2=0` and `P0.P2=1`, with each independent squared momentum `-1`.
The dependent external momentum also has squared momentum `-1`.
Its numerator's native loop momenta are fixed by stable edges 6, 8, and 12,
respectively, and agree with the historical `k1`, `k2`, and `k3` routing.
The massive three-point cards have `P0.P1=1/2` and independent squared momenta
`-1`, so the dependent third virtuality is also `-1`.

The six massive two-/three-point cases have no dedicated historical run cards.
`exhaustive_README.md` supplies graph-plus-kinematics CLI commands. Reference
`tests/test_integrals.py::test_dot_multiloop_two_and_three_point_examples_generate_finite_sector_sets`
checks finite sectors: respectively 4, 74, 6, 6, 74, and 117 reference sectors.
These counts characterize that decomposition and are not native sector-count
requirements. All six have no endpoint singular axes and no global Gamma pole
at epsilon zero. Their requested result is therefore a finite coefficient,
despite the padded negative-order display labels.

No stored independent numerical targets were found for the triple boxes or
these six massive families. An opt-in reference pySecDec comparison covers only
kite and self-energy, compares the finite term, and uses loose 512-point,
relative-tolerance-0.5 settings. It is a useful smoke reference, not a precision
acceptance target. The off-shell triple-box reference card explicitly omits a
default pySecDec target and notes that package generation can exceed its 30 GiB
watchdog limit.

## One-loop numerator cases

The three box numerator cards use massless propagators, null external legs,
`s=t=-1`, and maximum epsilon order zero. Historical loop momentum `k` equals
minus native `K(0)` on stable edge 7. The numerators are `2*k.p3`,
`k.k + 3*(k.p1)*(k.p2)`, and the original eight-term rank-five polynomial for
`box_high_rank_numerator`. The exact migration test checks the full polynomial,
not its filename or nominal rank.

The triangle numerator requires special care. Its historical explicit
propagators and scalar products imply external virtualities `{-1,0,-2}` at
`s=-1`; an unused historical `p2^2=0` annotation is inconsistent with those
propagators. The native card preserves the actual integral, using `P0^2=s`,
`P1^2=0`, and `P0.P1=-s/2`. It must not be replaced by the scalar triangle's
one-off-shell-leg point merely because both old fixtures referenced the same
kinematics file. This distinction is already documented in `examples/README.md`
and checked by exact propagator/numerator equivalence.

No integrated numerical targets are stored for the numerator fixtures.
Reference tests compare the triangle rank-one and box rank-two preliminary
Gaussian polynomials with pySecDec at selected Feynman parameters and epsilon
values; additional inline examples cover two-loop products and odd rank.
Those are algebraic checks, not integrated Laurent-vector targets. Native
`one-loop-reduce` followed by native `oneloop` masters is the next independent
validation route; its finite-order and degenerate-kinematics limits must be
respected as described in `hepkit-one-loop-native.md`.

## Explicit positive-orthant cases

| Native card | Complete density on the positive orthant | Maximum order | Historical target status |
| --- | --- | --- | --- |
| `issue_1.toml` | Seven variables, `F^(eps-2)`, prefactor one | 2 | Manual decimal values, no uncertainty or independent certification |
| `four_loop_hard.toml` | Nine variables, `U*F^(eps-3)`, prefactor one | 0 | Rounded Pathfinder full-sector QMC estimate with reported errors; not independently certified |

A textual comparison after removing whitespace verified that all three native
polynomial files exactly match the reference cards: issue-1 F has 45 terms,
hard U has 70 terms, and hard F has 105 terms. This check used no symbolic
runtime. Hard U is negative on the positive orthant, but its exponent is the
integer one; this introduces no fractional-power branch ambiguity. Neither
case may be reinterpreted as a projective simplex or a unit cube.

The issue-1 historical target records
`c0=10.36927755`, `c1=99.74055923`, and `c2=761.8752944`, with asserted zero
negative orders. `minimal_fix.toml` and `optimal_qmc.toml` extend the same
integral definition. Their differing integration schedules, lattice settings,
and evaluator backend are steering variants, not additional scientific inputs.

The hard historical full-sector target records
`c[-2]=-3.60617208198`, `c[-1]=-16.6719719507`, and
`c[0]=-149.867746145`, with reported standard errors approximately
`9.73e-7`, `6.84e-6`, and `5.92e-5`. Its report uses 3728 sectors and boundary
support grouping. Those provenance details are retained in
`examples/targets/four_loop_hard.json`; they do not establish independent
accuracy or require an identical native sector decomposition.

The hard all-sector FSD card requests 32 shifts and two million lattice points,
uses a real JIT evaluator, and disables precision guards. Its native pySecDec
counterpart requests one million maximum evaluations. The two `psd2807` cards
integrate only raw reference secondary sector 2807. They are partial-domain
benchmarks and cannot be compared with a native full-integral result or a native
sector with the same numerical index. A separately certified correspondence
between maps is required for any such partial-sector comparison.

## Comparison controls and evidence

The reference real O2 evaluator was observed during the broader project audit
to give incorrect box values while eager and complex O2 evaluations agreed with
the independent target. A parity run must therefore select a verified reference
backend explicitly. Historical O3, eager, disabled-guard, or partial-sector
timings are not automatically matched performance baselines for native portable
SymJIT O2 with precision checks.

Primary evidence resides in the ignored reference checkout:

- `examples/graphs/*.dot` and their kinematics YAML files;
- `examples/runs/dot_triple_box*.yaml`, `four_loop_hard*.{yaml,toml}`, and
  `issue_1/*.toml`;
- `src/pysecdec_bridge.py::_make_loop_integral` and `build_dot_bundle`;
- `src/formatting.py` global-prefactor convolution;
- `tests/test_integrals.py`, numerator comparisons near lines 5069–5158 and
  massive multiloop comparisons near lines 5440–5555;
- `examples/outputs/example_hard_from_polynomial.md` and the three saved scalar
  graph target JSON files.

The native durable evidence is the shipped run cards, polynomial files, target
provenance JSON, `examples/README.md`, and tests in `example_inputs.rs`,
`hepkit_one_loop.rs`, and the CLI all-card loading regression. The parity audit
itself ran no reference integration and made no fixture changes.
