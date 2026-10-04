# Native family parameterization entry

`ParametricIntegrand::from_family` exposes the existing Gaussian parameterization
for a borrowed native `IntegralFamily`, strictly positive powers, an already
weighted scalar numerator, caller-owned parameter symbols, regulator and target
dimension. `from_graph` contracts the native graph numerator/projector/overall
factor and multiplies the exact measure multiplier once, then delegates. The
scalar-only `ScalarParametricIntegral::from_graph` route is unchanged. No graph
rewriting, automatic partial fractioning, new physical graph type or new algebra
is introduced.

## Reuse evidence

The public API and implementation of FeynKit's `IntegralFamily::partial_fraction`
return exact coefficients and signed powers in original family order. Its
bounded algorithm owns repeated/affinely dependent denominator reduction;
exceeding its state limit returns an error. `IntegralFamily::sector` already
projects positive-power denominators while retaining the native loop variables,
external variables and kinematics. The native one-loop reducer's
`shared_family.rs` uses these conventions too; negative powers remain numerator
factors, not discarded data. FastSecDec does not reimplement either operation.

Native Symanzik construction and its `validate_labels` reject duplicate labels
and Schwinger labels occurring in source denominators, including physical mass
coefficients. The common FastSecDec `ParametricIntegrand::new` validator handles
distinct symbols, regulator separation and nonempty projective parameters.
Native `Atom::contains` handles the remaining weighted-numerator and dimension
collisions. These checks occur before Gaussian work or an early zero-numerator
return. Powers must have the family length and be strictly positive, preventing
zero powers from reaching the existing `power - 1` expressions.

The independent reviewer identified dimension-dependent physical coefficients
as a public-boundary ambiguity: native U/F may contain the tensor dimension
symbol, while the old Gaussian helper replaces that symbol in numerator
coefficients. Changing dimension while leaving a mass squared equal to the old
symbol would leave stale data. The entry now rejects that case with a typed
parametric error after native Symanzik construction. The same-symbol target
dimension is allowed; an unrelated physical mass symbol is untouched.

The pinned native family API has sector projection and momentum/power mappings,
but no family-wide coefficient specialization method. `Kinematics::apply`
applies existing momentum/scalar-product assumptions; it cannot decide whether
a physical mass named by the tensor-dimension symbol should itself change.
The caller must explicitly specialize that physical input before constructing
the family. This slice adds no family-copy algebra or implicit reinterpretation.

## Executable evidence

Four public API tests pass in `tests/family_parametric.rs`:

1. A duplicated massive tadpole with powers two and three is reduced by native
   partial fractions and sector projection to one propagator with power five.
   The original and reconstructed rational denominator products are canonically
   equal without expanding them. With weighted numerator six times that
   denominator, the existing Gaussian moment gives two at dimension two.
   Symbolica's native polynomial antiderivative independently integrates the
   original two-parameter simplex to the same answer, checking the measure and
   raised-power normalization as well as the projection.
2. A native bubble with numerator five, overall factor two, projector three and
   measure multiplier seven has total weight 210 exactly once. The family and
   graph entries agree, and the result equals 210 times the unchanged scalar
   route. Caller parameter names ending in underscores remain literal symbols.
3. Missing/zero/mismatched powers, duplicate parameters, regulator aliases,
   numerator/dimension aliases and a physical mass/Schwinger-label collision
   reject explicitly, including a zero numerator that must not bypass admission.
4. A propagator mass squared equal to the native dimension rejects a changed
   target dimension, admits the same formal dimension, and does not cause an
   unrelated physical mass symbol to be replaced.

The initial weight test compared differently represented affine exponents
(`2-(4-2*eps)` versus `-2+2*eps`) through rational cancellation, which does not
establish equivalence of their symbolic powers. The test now normalizes only
those small exponents through native expansion, keeping the density factored
and the exact factor 210 requirement unchanged. No production expression was
expanded or modified to satisfy the test. The first failed log is retained as
`output/native-family-entry-first-attempt.log`.

The focused gate
`cargo test --locked -p fastsecdec --test family_parametric --lib --test native_input -- --test-threads=1`
passed **49 tests**: 31 library, four new family tests and 14 native-input tests;
three pre-existing expensive library probes remained ignored. Final evidence is
`output/native-family-entry-tests.log`. Independent review is recorded in
`native-family-entry-independent.md`; the coordinator runs the combined
workspace/lint gate separately.

## Scope and next diagnostic

Different partial-fraction branches can have different active parameter lists;
they must be parameterized separately rather than concatenated into one
`ParametricIntegrand`, which has a single common parameter list. This entry does
not choose how to combine independent branches or change the original graph's
provenance.

The separate triple-box topology probe already verified native reduction of ten
denominators to eight with powers `[1,1,2,2,1,1,1,1]`, one coefficient-one term,
unchanged three-loop kinematics and exact canonical rational-product equality.
That proof motivates a bounded projected-family cost/full-vector diagnostic;
it does not certify its convergence or change default graph preprocessing.
See `triple-box-offshell-diagnostics.md`. Any such diagnostic uses the public
native operations above and retains its complete scope, covariance, precision
and resource outcomes.
