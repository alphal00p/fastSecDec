# Compact affine projective preparation

This slice prepares the native projective delta measure for GCAD. It does not
implement endpoint resolution, analytic continuation, threshold kernels or a
new integral domain in the existing sector generator.

`AffineProjectivePreparation` retains the original `ParametricIntegrand` and
eliminates one declared coordinate, by default the last. The surviving original
symbols keep their order, and the eliminated image is
`x_d = 1 - sum(x_i, i != d)`. The target domain is the open simplex defined by
positivity of each survivor and this complement. The same polynomials retain
the closed boundary geometry for later endpoint certificates.

## Native reuse and measure

The existing projective input constructor already checks distinct coordinates,
coordinate-independent prefactors and exponents, and homogeneous density degree
minus the parameter count. Existing primary-sector maps supply a different
gauge; they do not supply this compact affine preparation. The new constructor
uses Symbolica's native literal `Replacement` and `replace_multiple` on each
factor, including polynomial numerators. It neither expands the full density
nor reconstructs a polynomial algebra or graph.

All original terms, complete exponents and branch roles remain present. A
nonzero eliminated monomial power becomes a positive complement factor with the
same exponent. Original factor and monomial indices remain explicit. No
auxiliary regulator is introduced: any family supplied before the gauge must
already satisfy projective homogeneity, or needs its own compensated-family
proof. Later auxiliary continuation remains separate.

The unit measure is a **delta measure**, not Euclidean surface area. Introduce
`c = 1 - sum(x)` and use `(surviving coordinates, c)` as square coordinates. The
inverse has `x_d = 1 - sum(survivors) - c` and determinant of absolute value one.
Integrating `delta(c)` therefore leaves the ordinary survivor measure. The
native Matrix determinant probe verifies every elimination in dimensions one
through three. A separate tangent Gram determinant equals three in the
three-coordinate example, explicitly distinguishing surface area from this
delta measure.

## Request identity and boundaries

`GcadRequest::projective` consumes the typed preparation or retains a supplied
`Arc`. It shares the preparation's existing original-input owner instead of
copying either density. `input()` continues to
refer to the original projective density; `prepared_terms()` and
`domain().coordinates()` describe the transformed factorized density. Request
identity includes both owners, the map, elimination choice, factor origins,
domain and measure. Signed factors carry both original and prepared native
polynomials. Equal split geometry alone cannot authorize reuse of a proof for
a different density or gauge.

The kinematic-role check fences every original integration coordinate,
including the eliminated one. Exact rational specialization remains in the
existing GCAD adapter. No preparation is mislabeled as a unit cube or positive
orthant. A one-coordinate projective input produces a valid zero-dimensional
preparation; the GCAD entry explicitly declines this exact-only case instead of
inventing a stochastic sector.

## Validation status

The native API probe passed two controls against the same Symbolica `74225696`
graph as the initial adapter gate. It verifies simultaneous substitution with
complete epsilon/auxiliary powers and the square delta-measure identities.
Evidence is retained under `target/no-deformation-projective/`.

Six integrated controls cover all elimination choices for a multi-term complex
density; the delta/surface distinction; a native HEPKit physical massive bubble
and both causal signs; foreign density/gauge evidence and eliminated-coordinate
role rejection; zero-dimensional/wrong-domain handling; and shared owner/term
pointers with surviving request lifetime after the original handles are dropped.
All six passed against `c540d3f` in 0.08 seconds, recorded in
`target/no-deformation-projective-tests-r4.log`. The first integrated run exposed
two test assertions that compared undistributed native expressions structurally;
their correction uses native expansion before exact equality, without a
tolerance or production arithmetic change. The combined native gate then passed
449 core tests with 20 existing ignores, eight coefficient controls, 16 public
generation controls and four native API probes; see
`target/no-deformation-combined-native-tests-r4.log`. Strict scoped Clippy also
passed in 67 seconds (`target/no-deformation-combined-clippy.log`).
General closed-face density regularity,
exceptional-stratum completeness and endpoint continuation remain outside this
preparation certificate.
