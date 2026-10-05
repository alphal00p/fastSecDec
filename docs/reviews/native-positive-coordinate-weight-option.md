# Conditional native composition of coordinate weights

This is source-only research, conditional on the phase trace identifying
coordinate-weight Series products as a material cost. The current timeout has
not established that diagnosis. No executable proof, production change or
speedup is claimed.

The named experiment presently expands each surviving factor `x_i^p_i(eps)`
separately, then multiplies its native Series into the formal regular Series.
All surviving coordinates belong to the open unit cube, and the existing
endpoint admission requires affine regulator powers. Thus the scalar identity

`product_i x_i^p_i(eps) = exp(sum_i p_i(eps)*log(x_i))`

holds on that declared positive domain. It does not justify a rewrite for
negative or complex bases, arbitrary residual polynomials, or evaluation of
`log(0)` at an endpoint. Subtraction and face evaluation must remain before
this scalar-weight assembly, as in the current loop. Boundary-eliminated axes
remain omitted. No positivity inference from sampled values is involved.

## Existing native owner

The public `Series<AtomField>::exp()` in Symbolica `src/poly/series.rs:1260`
already owns exponential coefficients, factorials, native Series products and
truncation. It rejects essential singularities and handles its constant term
through native exponential/log normalization. The existing native
`derivative::tests::series_exp_log` exercises the public Series operation.
`AtomField` ring multiplication in `src/domains/atom.rs` uses ordinary Atom
products and optional native normalization, without blanket polynomial
expansion. This makes a small proof of compact powers of a summed logarithmic
coefficient reasonable; it does not establish actual large-input cost.

A narrow candidate can construct the scalar logarithmic sum with native Atom
operations, request its native Series through the existing expansion cache,
then call that Series' `exp()` and use the existing guarded `scale` exactly
once. No handwritten convolution, factorial, Laurent coefficient, derivative,
new function callback or native patch is needed. The scalar-independence check
must still exclude every named regular coefficient from the scalar operand.

Constructing an `exp(sum)` Atom before requesting its Series is not equivalent
as a representation choice: `normalize_fun` turns exp into `E^sum`, and
`normalize_pow` invokes `simplify_exp_log`, which may turn grouped logs back
into products or powers. Calling the existing **Series** exponential directly
avoids that initial whole-expression rewrite while retaining native constant
normalization. Native definitions and exact IR remain the downstream owners.

## Bounds and admission

The logarithmic sum can have leading regulator order zero or one, whereas its
exponential has a nonzero constant on the positive open domain. A requested
relative width is therefore not itself proof of the resulting absolute
remainder. Preserve the current native absolute-bound check, checked deficit
retry and iteration limits without inferring a replacement pole budget.

With no active factors, retain the original identity operation. If native Atom
normalization proves the complete logarithmic sum exactly zero, likewise skip
the weight multiplication. Do not request a finite-depth zero Series and infer
its cutoff: the previously audited native zero series can reset its bound to
zero. Unknown symbolic cancellation must keep native bounds and failure behavior;
it cannot be dropped because one numerical evaluation vanishes. Constant
zero-slope powers and all coordinate signs still need the ordinary native
admission and coefficient controls.

## Small proof required before an actual experiment

Compare the candidate with the unchanged sequential native factor expansion
over all available orders and actual native remainders. Cover two and several
positive coordinates; mixed positive, negative and rational affine slopes;
zero slopes; an empty active set; exact-zero logarithmic sum; and negative
requested maxima with a regulated Gamma pole. Retain native unsupported-input
errors. Small coefficient equivalence may use native expansion where compact;
do not force a giant actual expression to expand just to compare factorizations.

The existing Taylor and IBP small controls should retain exact complete-vector
agreement. Then use source-bound exact rational point checks at 512/1024 bits,
the rounded-coordinate weighted/forced-replay checks, unchanged conservative
cancellation metadata and callback-free native IR reload. All these checks are
separate from a later bounded actual representative and its independent
original-expression oracle. No numerical result or conditioned error guarantee
follows from the algebraic identity alone.
