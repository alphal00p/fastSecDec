# Independent review: native integral-family parameterization entry

This reviews the public `ParametricIntegrand::from_family` seam independently of
its author. It is a native object boundary, not a new reduction or Gaussian
implementation. Final source and the author's focused execution log were reviewed. No blocking
finding remains after the dimension-coefficient admission fix. Source findings
below distinguish the existing helper from the newly public contract.

## Ownership and normalization

`from_graph` retains the established native scalar contraction through
`GraphIntegral::scalar_numerator(default_algebra_settings())`, multiplies the
explicit measure multiplier once, and delegates to `from_family`. The latter
calls the existing Gaussian helper without reapplying graph weights, projectors
or measure multipliers. Native `IntegralFamily` owns momenta, scalar-product
basis, kinematics and ordered inverse propagators. Parameters remain native
`Symbol` and expressions remain native `Atom`; no text serialization, custom
graph or alternate algebra is inserted between these objects.

The public documentation distinguishes the already weighted scalar numerator
from the normalized Minkowski measure and Feynman-parameter Gamma/sign factors.
The graph test uses separate numerator5, overall2, projector3 and multiplier7,
checks the native scalar value210, then compares delegated and direct-family
expressions against210 times the unchanged scalar graph density. This exercises
the important exactly-once weight boundary instead of merely comparing two
calls through the same wrapper.

## Native projection and admission

FeynKit `integrals/partial_fraction.rs::partial_fraction` returns coefficients and
signed powers in original denominator order. FeynKit `integrals.rs::sector`
retains positive-power denominators in that order, leaving loop and external
variables unchanged. The new entry accepts strictly positive unsigned powers;
zero, wrong-length and parameter-count inputs are rejected before Gaussian work.
Its documentation correctly leaves branch selection, negative-power numerator
factors, coefficients and separate parameter lists to the caller. It does not
silently partial-fraction a graph, shift loops or interpret an empty projection
as proof of zero. Existing native scalelessness certification remains the only
zero-integral shortcut.

The exact test uses native partial fractions and projection to combine repeated
massive denominators with powers2+3 into one power5. A weighted numerator6D
cancels one denominator algebraically. At dimension2 the normalized answer is2;
the original two-parameter representation is also integrated using Symbolica's
native polynomial antiderivative. This is independent mathematical evidence for
the public seam and does not introduce a test quadrature implementation.

Parameter/regulator distinctness reuses `ParametricIntegrand::new`. Collisions
with numerator, target dimension and native dimension are rejected explicitly;
family-expression collisions remain owned by FeynKit `validate_labels`, reached
through `symanzik`. The source check confirms `validate_labels` checks label shape/uniqueness and native scalar-product/denominator occurrence,
rather than relying on naming rules.
Tests include literal parameter symbols ending in underscores, duplicate labels,
regulator collision, a physical mass equal to the integration label and invalid
powers even when the weighted numerator is zero.

## Resolved admission finding

The review found that the original newly public documentation promised literal
symbolic-dimension substitution, while the existing helper replaced that symbol
only in numerator coefficients. Native base and auxiliary U/F can retain the
symbol if a caller uses it in a physical propagator/kinematic coefficient, such
as mass-squared equal to the tensor-dimension symbol. This predates the wrapper
but is directly reachable through a native family.

The author added a narrow guard after native Symanzik construction: when the
requested dimension changes, a remaining native dimension symbol in U/F is a
typed admission error with an instruction to specialize those coefficients
explicitly. The same formal dimension remains allowed; an unrelated physical
mass symbol remains unchanged. The new regression covers all three cases and
passed. This resolves the stale-symbol ambiguity without introducing implicit
family specialization or another Gaussian algorithm.

No worker pool, integration loop, sampling rule or checkpoint policy is introduced
by this API. Future HEPKit callers can retain their native family and exact weighted
numerator while steering reduction and integration themselves. The seam neither
adds a Python bridge nor promises automatic graph reduction.

## Reviewed execution evidence

The focused author gate passed **49 tests**: 31 library tests, four public-family
tests and 14 native graph/input tests; three pre-existing expensive library
probes stayed ignored. The reviewer inspected
`output/native-family-entry-tests.log`, the final dimension guard/regression and
the graph weight proof. The latter normalizes only small affine U/F exponents
through native expansion before comparing factored densities, because rational
cancellation alone cannot identify differently represented symbolic exponents.
It does not expand the full density or weaken the exact factor210 assertion.
No independent symbolic rerun was started during the coordinator's combined
gate. The coordinator owns final workspace/lint acceptance.
