# Six massive multiloop fixtures

This is the preparation record for a bounded production-CLI diagnostic and its
subsequent independent numerical checks. No independently certified values are
claimed here. Raw status streams, artifacts, checkpoints and coefficient vectors
belong under ignored `output/diagnostics/massive-multiloop/`.

## Scientific inputs

All six shipped cards use the native scalar model, the native massive parameter
card (`UFO::mt = 1`), unit propagator powers, dimension `4-2*eps`, explicit measure
multiplier `1`, and requested highest Laurent order zero. They use normalized
per-loop measure `d^Dk/(i*pi^(D/2))`; the native scalar parameterization supplies
`(-1)^N Gamma(N-LD/2)`. All propagator masses are one and all external virtualities
are minus one. Three-point cards set `P0.P1 = 1/2`, with the third momentum fixed
by native momentum conservation. They request ordinary domain certification,
without a no-threshold assertion.

| Native card | Loops | Propagators | External invariants | Expected sign |
| --- | ---: | ---: | --- | --- |
| `kite_2loop.toml` | 2 | 5 | `p^2=-1` | Negative |
| `self_energy_3loop.toml` | 3 | 7 | `p^2=-1` | Negative |
| `three_point_2loop.toml` | 2 | 5 | `p1^2=p2^2=p3^2=-1` | Negative |
| `three_point_2loop_6line.toml` | 2 | 6 | Same | Positive |
| `three_point_3loop.toml` | 3 | 7 | Same | Negative |
| `three_point_3loop_8line.toml` | 3 | 8 | Same | Positive |

The expected finite support is `[eps^0]`. The historical generation tests found
no singular axes for these inputs; their padded negative-order output arrays are
not evidence of actual poles. Native output need not preserve that padding or the
historical sector counts. Exact graph and convention parity is also recorded in
`scientific-fixture-parity.md`.

## Existing reference evidence and reuse

At Pathfinder revision `582d8c7f6dde9bf750750d4c2a2d85a94ce940cd`,
`tests/test_integrals.py::test_dot_multiloop_two_and_three_point_examples_generate_finite_sector_sets`
covers generation of all six cases. The optional
`test_optional_multiloop_fsd_low_stat_compare_to_pysecdec` covers only the kite and
self-energy. It generates a separate pySecDec package at 512 maximum evaluations
and relative tolerance `0.5`, and uses a loose eight-times-summed-error comparison.
There is no stored precision target in that test. It is useful independent
low-statistics evidence once actually executed, not a convergence certificate.

The existing reference CLI supports `--dot-engine pysecdec` and already owns the
package-generation and integration bridge. Its invocation can be confined to the
ignored reference environment; no Python implementation or runtime dependency is
added to FastSecDec. Preserve the existing graph/kinematics files and explicitly
select massive mode, mass one and the pySecDec normalization. Since these inputs
are finite, epsilon-dependent normalization conversions do not mix any pole into
the finite coefficient, but the explicit measure and sign must still agree.

The native HEPKit OneLOop provider supplies A0/B0/dB0/C0/D0. The native
`one-loop-reduce` shared-family admission explicitly requires exactly one loop.
Neither provides these multiloop values. Native Vakint's MATAD/FMFT routes are
vacuum-specific; its general unrecognized topology constructor selects
`EvaluationOrder::pysecdec_only(None)`. These off-shell nonvacuum integrals
therefore have no newly discovered pure-Rust analytic master reference through
those interfaces. Reimplementing a reduction or a numerical integrator is not
justified by this limitation.

## Bounded native diagnostic

The ignored `fastsecdec-cli` test `massive_multiloop` invokes the production CLI
serially for generation, then integrates all successfully generated artifacts.
Each stage has a 180-second watchdog. The numerical allocation is 1024 lattice
points times eight independent shifts per kernel, seed `20261004`, two workers,
Korobov-3 periodization and one refinement round. It saves the complete production
vectors and covariance, numerical diagnostics, structured progress, generation
timings, dependency provenance and resumable checkpoints. Every case is attempted
and a summary is rewritten after each stage, including failures.

The checks require finite real output, the normalized Euclidean sign, complete
production coverage, consistent covariance dimensions and no evaluation failures.
They do not compare sector counts, assume sector numbering, or turn an incomplete
estimate into zero. Passing these checks establishes executable pipeline coverage
only. Independent pySecDec comparisons and increasing-work/independent-seed
convergence checks remain separate gates.

Run explicitly after the current dependency/runtime validation gate:

```sh
cargo test -p fastsecdec-cli --locked --test massive_multiloop -- \
  --include-ignored --test-threads=1 --nocapture
```

Set `FASTSECDEC_MULTILOOP_OUTPUT` to save a distinct diagnostic campaign. Preserve
the artifact and checkpoint identities when performing follow-up integration.
