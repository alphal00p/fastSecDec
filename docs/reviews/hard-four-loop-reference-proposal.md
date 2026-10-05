# Ordinary F-only hard-four-loop reference proposal

The original hard-four-loop reference draft has never executed. Replace that
planned route with the existing ordinary pySecDec package generator and its
supported `other_polynomials` channel. The input remains the complete
nine-dimensional positive-orthant integral `U*F^(eps-3)`, through epsilon zero,
with unit prefactor, no selected sector and no projective constraint. Nothing
from the completed FastSecDec sector or coefficient representation enters the
reference engine.

The exact supplied exponent of U is the fixed integer one. It has no fractional
or regulator-dependent power requiring a monomialized residual. The full signed
polynomial U must still be transformed and included, including its monomial
growth at infinity. Moving it to `other_polynomials` does not remove it from
the density or assume it is bounded after the orthant maps.

## Native API and source evidence

Reuse the frozen reference `FSD.parse_args`, `FSD.build_request` and
`FSD.validate_request` for
`examples/runs/four_loop_hard_all_sectors_pysecdec_native.toml`, overriding only
the one worker, fresh output directory and explicit requested order zero.
These functions own inherited-card loading and request validation. Then reuse
`uf_topology.uf_topology_data_from_request` and the existing bridge's
`_uf_decomposition_polynomials` for its source strings. Do not create a second
YAML/card parser, polynomial parser or affine-exponent routine.

Preflight must require the expected original ordered nine variables, no physical
parameters, no sector filter, zero parameter measure powers, U exponent exactly
one, F exponent `eps-3`, and global prefactor exactly one. Retain both inherited
external cards and native card/U/F hashes. Check the two supplied polynomial
strings against the already audited native inputs; a changed source is an error.
The two returned bridge polynomials are `[U, F^(eps-3)]` for this admitted case.

Call public `pySecDec.code_writer.make_package` directly with:

```python
make_package(
    name="hard_four_loop_together",
    integration_variables=source.x_names,
    regulators=["eps"], requested_orders=[0],
    polynomials_to_decompose=[polynomials[1]],
    other_polynomials=[polynomials[0]], prefactor=1,
    decomposition_method="geometric_infinity_no_primary",
    normaliz_executable=recorded_native_normaliz,
    processes=1, form_threads=1, form_work_space="50M",
    ibp_power_goal=-1, split=False,
    use_iterative_sort=False, use_light_Pak=False,
    use_Pak=False, use_dreadnaut=False,
    pylink_qmc_transforms=["korobov3x3"],
)
```

The installed API documents `other_polynomials` as additional factors without
fan construction (`code_writer/make_package.py:1505–1528`), and documents
`geometric_infinity_no_primary` as integration over the full positive orthant
(`make_package.py:1637–1655`). There is no primary simplex reduction.
Its geometric decomposition constructs the hull from `sector.cast` only, then
applies the same transformation to `sector.other`
(`decomposition/geometric.py:192–209,229–230`). The code generator factorizes
each transformed other polynomial and includes its extracted monomial in the
subtraction powers (`code_writer/make_package.py:1037–1050,1085–1090`). Thus U's
signed value and possible mapped negative powers remain owned by the native
engine. No FastSecDec implementation of that transformation or subtraction is
needed. Native parsing also checks that the extra factor exponent is independent
of integration variables (`make_package.py:214–225`).

These are source/API checks, not an executed equivalence test or a promise that
the difficult package will finish. The first bounded execution must retain the
actual generated arguments, all-sector metadata and full prefactor/physical
tuple for independent review. There is no reason to spend an initial full U/F
fan run just to preserve an unexecuted draft's anticipated extra work.

## Bounded execution and uncertainty ownership

After the projected scalar/rank-two reference slots, use a new
`hard4loop-together-attempt-1` directory, with 600 seconds total, 30 GiB
process-tree RSS, one allowed CPU and at most 180 remaining seconds for numerical
work. Reuse the reviewed reference watchdog/recording/import-guard helpers.
Generation is a separate process requiring the exclusive Symbolica slot; FORM,
C++ and numerical subprocesses run under the fail-closed guard afterward.
Do not silently extend any bound or overwrite the old unexecuted script.

Build the ordinary `*_pylink.so` and call existing `IntegralLibrary` with
`together=True`, explicit Korobov3, no fit function, `cbcpt_dn1_100`, N8311/R32,
seed20261219, one numerical thread, `maxeval=265952`, `epsrel=0.01`,
`epsabs=1e-12`, and verbose allocation logs. This is a prespecified initial
observation; counts require actual logs and no accuracy target is asserted from
the request. Retain every returned physical coefficient and stated uncertainty,
the original complete string tuple before native conversion, and all failed
stage evidence. The ordinary sector-sum path avoids the known disteval
shared-shift covariance omission without a new estimator. Any joint covariance
between different epsilon orders remains unknown.

This proposal has no executed wrapper or scientific outcome yet. Independent
source review and coordinator runtime assignment are required before launch.
Production FastSecDec remains Rust-only and independent of this provider.
