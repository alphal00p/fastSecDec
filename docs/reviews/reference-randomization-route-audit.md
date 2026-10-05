# Reference randomization and uncertainty routes

This follow-up source/evidence audit distinguishes the actual transports of the
eight frozen external references. No new symbolic process, estimator or
numerical integration was run. The issue1 shared-shift variance finding does
**not** establish the same defect in the six massive fixtures or double box.

| Fixture | Actual completed external route | Random-shift ownership | Typed disposition |
|---|---|---|---|
| kite_2loop | DOT → generated sum package → IntegralLibrary / pylink | One advancing native C++ QMC engine | Retain Checked |
| self_energy_3loop | Same; retained serial-generation retry | Same | Retain Checked |
| three_point_2loop | DOT → IntegralLibrary / pylink | Same | Retain Checked |
| three_point_2loop_6line | DOT → IntegralLibrary / pylink | Same | Retain Checked |
| three_point_3loop | DOT → IntegralLibrary / pylink | Same | Retain Checked |
| three_point_3loop_8line | DOT → IntegralLibrary / pylink | Same | Retain Checked |
| double_box | DOT/LoopIntegral package; copied-package native IntegralLibrary continuation | Advancing native C++ QMC engine, explicit seed 20261203 | Retain Checked |
| issue_1 | Direct U/F → generated disteval package | Each coefficient kernel restarts NumPy RandomState(0) | Unverified; pulls ineligible |

## Six massive records

Every retained successful raw report has `request.topology_source="dot"` and
`dot_engine="pysecdec"`. All six record `make pylink` compilation, ordinary
package loading, empty disteval kernel/sector result arrays, and no disteval
integration log. The self-energy record is the successful `.serial.raw.json`
retry. The file-level evidence matches the previously frozen raw hashes.

In the frozen checkout, `FSD.py:2424–2429` routes native DOT input to
`pysecdec_bridge.py::run_pysecdec_package` (1596–1628). That function constructs
`IntegralLibrary` and invokes its generated library. The direct-U/F all-sector
route at `FSD.py:2474` instead calls `run_pysecdec_uf_all_sectors` and disteval.
The presence of disteval support elsewhere in the same bridge is not evidence
that a particular DOT report used it.

Each massive package contains one constituent integral. Its generated
`*_weighted_integral.cpp` constructs **one** shared native QMC integrator before
converting all sector/order integrands, and passes that same shared object to
each `secdecutil::amplitude::QmcIntegral`. Native `amplitude.hpp:131–174` calls
the stored object's integration method on successive computations, without
re-seeding or copying the engine per sector. Installed `qmc.hpp:2649–2655`
draws shifts from the object's advancing `randomgenerator`; its constructor at
3421 initializes that engine from `std::random_device`.

The Python wrapper's zero default seed leaves that initialization intact:
generated pylink code seeds the engine only for nonzero seed. The bridge does
not forward its general request seed. Likewise, the wrapper's default
`number_of_threads=0` is normalized to one outer amplitude worker
(`amplitude.hpp:516–520`); this serially advances the shared engine between
sector computations. Native QMC can internally parallelize a sample batch,
but its shift array is generated from the one engine before that batch. The
reference `--workers` setting controls package generation, not this amplitude
worker count.

Consequently equal dimensions do not cause each sector to restart an identical
random stream on this route. The full projective dimensions are 4, 6, 4, 5, 6
and 7, respectively; numerical integrands may have fewer active coordinates.
Possible later refinement calls continue the existing engine. The raw bridge
did not preserve actual refinement histories, selected lattices, realized seed
or aggregate counts, so those remain unknown. This source check rules out the
**identified disteval restart mechanism**, not every possible statistical
limitation. Existing finite-error and non-convergence qualifications remain.

## Double box and issue1

The successful double-box continuation explicitly imports IntegralLibrary,
loads the retained graph-generated sum-package library and supplies seed
20261203, one numerical thread and one amplitude thread. Its physical native
tuple and generated Gamma prefactor were separately audited. It also has one
constituent integral and uses the advancing C++ engine. No disteval-based
downgrade follows from this review.

Issue1 is different in the observed native route, not merely in its topology.
Its installed disteval scheduler constructs every kernel RNG with seed zero,
and its equal-dimensional initial lattices use common shift sequences. The
reported amplitude variance sums marginal variances without their covariance.
Source/normalization/transport checks therefore remain in provenance, while
the fixture is typed Unverified and comparison eligibility is false. Its
reported errors are retained unchanged, with no reconstructed uncertainty.

No blanket fixture downgrade or new implementation-mirroring test is justified
for the other seven records. Existing transport tests preserve their actual
route provenance; the new issue1 regression specifically enforces its
Unverified/ineligible boundary. Further independent uncertainty calibration or
matched convergence campaigns remain separate work for every reference.
