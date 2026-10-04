# Frozen Pathfinder QMC rules and refinement

This read-only source audit compares the frozen experimental reference with the native Kuo33002 lattice investigation. It does not change a rule, transform, estimator or stopping condition. Paths below are relative to `DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder` unless stated otherwise.

## Actual default rule

`FSD.py:865–874` defaults to `pysecdec-default`, an independently evaluated rank-one rule obtained from the published CBC/PT `cbcpt_dn1_100` vector table. It does **not** default to QMCPy's base-two lattice. `src/qmc_lattice.py:115–145` selects the smallest available table size at least as large as the requested count and takes the vector's dimension prefix. The table supports up to 100 dimensions. The reference reads installed `pySecDecContrib/include/qmc.hpp`; this table lookup is a reference-only dependency and is not proposed for the native implementation.

The bundled smaller table at `src/qmc_lattice.py:18–28` agrees with the corresponding full-table entries. With a sufficiently high or disabled request cap:

| Requested count | Actual prime count | First five generating-vector entries |
| --- | --- | --- |
| 4096 | 4261 | 1, 1648, 1902, 1757, 1533 |
| 8192 | 8311 | 1, 3068, 1811, 1128, 1964 |

The 8311 entry is also present at `DO_NOT_PUSH_FOR_REFERENCE_ONLY/pysecdec/pySecDecContrib/qmc.hpp:1407`. `src/qmc_lattice.py:315–347` explicitly exposes the difference between requested and concrete point counts. The independent point construction at lines 34–46 is `(index*z mod n)/n`, followed by the random shift modulo one.

There is an additional default cap: `FSD.py:845–852` sets `qmc_max_samples_per_iter=4096`. Despite the option's adaptive wording, `src/integrator.py:2662–2683` applies this cap before testing the refinement mode, so it applies to democratic runs too. An unqualified request for 8192 points therefore uses the 4261 rule. Requesting the 8311 rule requires disabling or increasing the cap. A comparison must record the concrete count and vector, rather than assuming that equal nominal requests select equal rules.

The optional `qmcpy` path (`src/qmc_lattice.py:154–193`) requires a power-of-two count and instantiates QMCPy's default `Lattice` with shifted replications; it does not supply a custom generating vector. The frozen CLI's ordering default for that path is `linear` (`FSD.py:878–885`).

## Transform, supports and replicates

The default periodization is Korobov alpha 3 (`FSD.py:856–862`). `src/integrator.py:940–962` implements the integrated beta-polynomial coordinate transform and the product Jacobian. Thus the transform family matches the current native comparison, but the lattice family does not.

The default support mode is `boundary`, not `full` (`FSD.py:888–902`). `src/integrator.py:1011–1092` can group complete Laurent coefficients by their actual coordinate support when prepared evaluator metadata exists, or reduce the deepest pole's endpoint support. It retains complete coefficients rather than sampling individual projector-source pieces. The comment at lines 965–983 explicitly identifies dummy-coordinate periodization weights as a source of unnecessary variance. QMC-specialized evaluators may fuse the transform and its Jacobian (`FSD.py:940–950`). These are separate differences from the generating vector and must be recorded in any matched experiment.

There are 64 shifted replicates by default (`FSD.py:815–821`). Shift vectors use NumPy's seeded generator (`src/qmc_lattice.py:49–53`). Democratic sampling and correlated sector sums are defaults (`FSD.py:906–937`). The correlated path uses shared shifts for matching support groups and sums sector values before the shift statistics (`src/integrator.py:3119–3147`, `3170–3190`).

## Refinement and safeguards

In democratic mode, `_qmc_iteration_request` keeps the configured capped point count and shift count fixed. Each iteration receives fresh seeded shifts. The aggregate shift statistics persist across iterations (`src/integrator.py:2810–2812`, `3170–3190`). Thus the default refinement increases the number of independent shifted estimates, not the lattice size.

Adaptive mode doubles the initial requested point and shift counts on each iteration until the configured maxima, rounding QMCPy counts down to a power of two (`src/integrator.py:2662–2683`). Its default initial counts already equal the default production caps: 4096 requested points and 64 shifts (`FSD.py:825–852`). Its sector scheduling ranks marginal error contributions, retaining at least 5% of sectors covering 80% of the summed contribution (`src/integrator.py:2630–2659`). This scheduler does not inspect lattice spectra.

The QMC stopping path uses estimated absolute/relative accuracy, minimum iteration count and time limits (`src/integrator.py:1960–1982`, `3190–3228`). Inspection of the rule selection, scheduling and stopping code found no explicit dual-lattice resonance test, comparison against an independent generating vector, or mandatory cross-size discrepancy check. Monte Carlo reliability diagnostics elsewhere in this file are not a QMC spectral safeguard.

## Implication for the current five-axis investigation

For the native rule reported by the numerical reviewer, `n=8192`, `z=[1,2443,739,1399,5901]` and `h=[1,1,1,1,2]` give `h·z=16384`, which is zero modulo `n`. The corresponding rank-one lattice average of that Fourier mode therefore does not cancel; random shifts randomize its phase. For the reference's 8311 vector, the same dot product is 9936, with nonzero remainder 1625. For its default capped 4261 vector it is 8374, with nonzero remainder 4113. This specific resonance is absent from both reference choices.

This arithmetic supports investigating a different prime rule or an independent vector as a controlled diagnostic. It does not establish that this mode alone explains the observed fixture's error, nor does it certify the reference rule for that integrand. Increasing shifted replicates can still reduce the uncertainty of a resonant mode even when increasing point counts along an unfortunate embedded sequence does not. Any comparison should hold the complete coefficient, periodization, support policy and arithmetic fixed, report actual rule provenance, and retain the measured shift covariance. No implementation change or performance acceptance claim follows from this audit.
