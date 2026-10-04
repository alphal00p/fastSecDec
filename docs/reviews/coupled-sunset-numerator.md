# Coupled two-loop sunset numerator

This closes the additional inline family at `FastSecDecPathFinder/tests/test_integrals.py:5096`, revision `582d8c7f6dde9bf750750d4c2a2d85a94ce940cd`. It is additional to the original fifteen historical DOT inputs, not a replacement for one of them. The new native graph and run card use the existing scalar model and massless parameter card.

## Native objects and routing

Both external edges point outwards. Three internal edges run from the first vertex to the second; stable edges 2 and 3 carry native loop-basis IDs 0 and 1. The native denominator list is exactly `K(0)^2`, `K(1)^2`, `(K(0)+K(1)+P(0))^2`. The numerator is a native vertex fragment, `K(0).K(1)+2*K(0).P(0)`, and `P(0)^2=-1` on the shipped card. Unit propagator powers and the normalized measure `prod d^Dk/(i*pi^(D/2))` are explicit. No additional Euler-gamma, scale, symmetry or model factor is inserted.

The exact native routing test passed. Its scalar control uses `FeynmanDiagram::with_numerator(1)` on that same diagram, followed by `GraphIntegral::new`; it does not manipulate DOT text or recreate routing. The parameterized density comes from `ParametricIntegrand::from_graph` and its existing `density()` method. Native Symbolica owns Gamma functions, series, substitution and evaluation. The tests introduce no reduction, Gaussian, symbolic-density or integration implementation.

## Independent pointwise density

The ignored script `output/probes/sunset_density_oracle.py` calls only `pySecDec.loop_integral.LoopIntegralFromPropagators` and SymPy. It reproduces the historical inline request, then retains the complete Gamma factor, U/F exponents, exact preliminary polynomial and 60-digit density samples. This is an external validation tool; no Python enters a Rust build or ordinary execution path.

The frozen oracle is `examples/references/sunset_2loop_numerator_density.json`, SHA-256 `d1838c19606fc86cc0bc63271457b043ffa6aac55658e33ab769740f42f03ae0`. It records pySecDec 1.6.6, SymPy 1.14.0, the exact request and script digest. U is `x0*x1+x0*x2+x1*x2`, F is `x0*x1*x2`, and its full common prefactor is `-Gamma(2*eps-2)`. Its density uses `U^(3*eps-5)*F^(1-2*eps)` times the recorded numerator.

Three rational simplex points are checked at epsilon zero, 0.11 and -0.19. All nine checks include the density divided by its common Gamma factor; the six nonzero-epsilon checks additionally include the full meromorphic density. Removing the common factor per native term before setting epsilon to zero avoids evaluating a removable infinity. These are pointwise parameterization checks, not integrated estimates or statistical uncertainty claims. Execution evidence: `output/sunset-density-pysecdec.log`.

## Independent integrated identity and normalization

[Grozin, *Higher radiative corrections in HQET*, Eq. (4.3)](https://www-library.desy.de/preparch/desy/proc/ali/proc/grozin_andrey/grozin_andrey.pdf) gives the massless bubble with explicit negative-propagator conventions. Composing that identity twice gives the scalar sunset. Changing three denominator signs to this repository's convention gives

\[
I(D,p^2)=-(-p^2)^{D-3}
\frac{\Gamma(3-D)\Gamma(D/2-1)^3}{\Gamma(3D/2-3)}.
\]

The finite control uses `D=5/2`, inside `2<D<3`; its Wick-rotated density and analytic value are negative. This checks the measure sign before meromorphic continuation to four dimensions. No unreconciled sign from another source is used as an oracle.

For the numerator, set `q1=k1`, `q2=k2`, `q3=-k1-k2-p`. Symmetry under permutation of the three equal massless denominators makes all pair and single-momentum integrals equal. Each integral with a numerator `qi^2` pinches a line and becomes a product of scaleless integrals. Therefore `6*integral(q1.q2)=p^2*I` and `3*integral(q1.p)=-p^2*I`. The requested numerator integral is `-p^2*I/2`. This is a narrow test identity, not a general multiloop reducer.

At `D=4-2*eps`, `p^2=-1`, the pole is `1/(8*eps)` and the finite coefficient is `13/16-gamma_E/4`. Tests use native Symbolica to expand the complete Gamma identity through epsilon one, at spacelike scales one and two. Every generated order is checked, including any cancelling lower pole; missing coefficients are never inferred from absent reference rows.

## Execution status

All four scientific tests passed in `output/sunset-scientific-tests.log`. The native published Kuo38005 rule uses 4096 points and 16 independent shifts for each numerator scale, complete sector coverage, full covariance and caller-owned weighted precision replay. The complete output orders are `[-1,0,1]`; no generated lower order or sector was discarded.

| Spacelike scale | Order | Native mean | Estimated standard error | Analytic value |
| --- | ---: | ---: | ---: | ---: |
| 1 | -1 | 0.124999999999887 | 1.51e-13 | 0.125 |
| 1 | 0 | 0.668196083772230 | 2.55e-12 | 0.668196083774617 |
| 1 | 1 | 2.53345226710478 | 3.31e-11 | 2.53345226713091 |
| 2 | -1 | 0.499999999999518 | 7.14e-13 | 0.5 |
| 2 | 0 | 1.97963715452945 | 1.26e-11 | 1.97963715453852 |
| 2 | 1 | 6.90899623009162 | 1.61e-10 | 6.90899623020526 |

The convergent scalar control initially used the same 4096 points and returned `-68.9172973421 +/- 0.0073816153`, consistent with the negative analytic value but failing the predefined precision cap. This evidence remains in `output/sunset-scientific-initial-inconclusive.log`. Increasing only that control's point budget to 65536, with the same sixteen shifts and unchanged tolerances, gives `-68.9339095487 +/- 0.0004342504`, versus `-68.9341406592`, a difference of 0.53 estimated standard errors. The fractional endpoint makes this control less smooth after the fixed Korobov3 transform; no tolerance or error estimate was loosened to accept it.

Seeds are 41821 for the scalar, 41823 and 41824 for the numerator scales. Complete covariance matrices are retained in the raw log. These checks establish the stated scalar sign and coupled-numerator identities at the selected points; they do not imply all-example convergence or matched performance parity. Independent native-object and physics review is recorded in [the separate review](coupled-sunset-independent.md).
