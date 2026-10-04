# Independent coupled-sunset review

This source review is independent of the fixture and test author. It covers the
native DOT/card, `tests/sunset_numerator.rs`, its caller-steered integration
helper, and the frozen external density oracle. Executable gate results belong
to the author's separate [campaign record](coupled-sunset-numerator.md).

The native graph uses two outgoing external edges and three parallel directed
internal edges. With the first two native loop-basis edges assigned `K(0)` and
`K(1)`, conservation at the first vertex gives the remaining momentum as
`-(K(0)+K(1)+P(0))`; squaring gives the requested third denominator. The explicit
test compares every native denominator and the complete scalar numerator to
that routing. Both degree-four vertices are supported by the existing native
scalar model. The run card supplies `P(0)^2=-1`, mass zero, `D=4-2*eps` and an
explicit unit measure multiplier, without a legacy routing or mass parser.

The numerator is a native vertex fragment. Constructing the scalar control with
`FeynmanDiagram::with_numerator(1)` reuses the same native topology and routing,
and the test checks that all denominators survive that operation. The local
density reconstruction was removed during review in favor of the existing
`ParametricIntegrand::density()` API. Small polynomial expansions in routing
assertions do not change production expression handling.

The frozen pySecDec oracle records the actual propagators, numerator, dimension,
kinematics, package versions and generating-script hash. Its nine rows comprise
three simplex interior points and three regulator values. The finite
Gamma-stripped density is compared at zero; complete densities are additionally
compared at both nonzero regulator values. Removing the common Gamma factor
from each native prefactor before evaluating zero avoids treating a removable
pole as a finite number. The oracle is pointwise independent parameterization
evidence, not an integrated reference or a numerical uncertainty estimate.

The integrated identity is consistent with the routing and measure. For
`q1+q2+q3=-p`, massless scaleless pinches and permutation symmetry give
`integral(q1.q2)=p^2*I/6` and `integral(q1.p)=-p^2*I/3`; therefore the specified
numerator integrates to `-p^2*I/2`. At `p^2=-s`, the two-loop scalar scaling is
`s^(1-2*eps)`, so the numerator result scales as `s^(2-2*eps)`. The test's native
Gamma/series expression has residue `s^2/8`, consistent with that identity.
The separate scalar check at `D=5/2` lies inside `2<D<3`, where this massless
off-shell sunset is both infrared and ultraviolet convergent, and checks the
negative sign of three positive-`q^2` propagators before Laurent continuation.

Both integrated scales use the existing native generation, portable O2 kernels,
weighted precision replay, published lattice and complete-vector accumulator.
The helper asserts full production and real components, and compares every
returned Laurent order through `eps^1`, including zero coefficients if exposed.
It implements no separate integrator, tensor reducer or master evaluator. This
closes a coupled two-loop scientific identity; it does not establish the next
triple-box cases or a matched convergence/performance campaign.

No scientific normalization, input-equivalence or native-reuse blocker was
found in this source review. The external oracle's SHA-256 is
`d1838c19606fc86cc0bc63271457b043ffa6aac55658e33ab769740f42f03ae0`.

The author subsequently reported all four scientific tests passing in 1.16
seconds, including the nine density rows and both complete Laurent vectors.
The convergent scalar check used 65536 points and 16 shifts after retaining an
earlier inconclusive lower-work observation; its result is 0.53 reported standard
errors from the analytic value. That is separate executable evidence, with the
original accuracy requirement retained rather than relaxed to fit the result.
