# Configurable coefficient field for fixed-variable polynomial conversion

Status (2026-10-06): the required API is upstream in Symbolica `community`
revision `58652fabc2f736302a570deaaf8d517679f7fe6e`. The local patch is removed;
FastSecDec uses that public source directly through Cargo.

## Historical proposal and validation

The adjacent local Symbolica patch adds
`AtomCore::to_polynomial_in_vars_with_field(variables, &AtomField)`. It reuses
the existing fixed-variable conversion recursion and propagates the supplied
coefficient ring. The previous method still uses `AtomField::new()` and retains
its behavior. This is a local dependency patch, not a published upstream change.

FastSecDec needs the degree of a factored regular numerator under a common
parameter rescaling. Native `is_polynomial(true, false)` admits polynomial
structure; conversion in one scaling variable leaves the physical parameters
inside compact expression coefficients. It avoids expanding their full support
merely to verify projective homogeneity.

The previous fixed-variable API hardcodes `AtomField::new()`, whose coefficient
zero test is statistical. A conservative scaling certificate must retain
unresolved coefficient cancellations. The new entrypoint allows
`statistical_zero_test: false`. It does not change differentiation, polynomial
arithmetic, or expression normalization.

The existing configurable `try_to_polynomial` is a different operation: it adds
encountered symbols to its polynomial variable map. `collect` and
`coefficient_list` use the same fixed-variable conversion as the old entrypoint.
The returned polynomial exposes its ring by immutable reference; `map_coeff` can
change the ring only after any coefficient has already been dropped. None
provides the required configuration at conversion time.

Three focused native controls cover a degree-10,000 compact coefficient,
unresolved cancellation retained with exact zero tests, and equality between the
old method and an explicitly supplied default field. FastSecDec controls cover
homogeneity, nonpolynomial rejection, exact support fallback, and the distinction
between a scaling identity and a nonzero certificate. Source review accepted the
patch. All three native controls and 28 affected FastSecDec/status controls
passed, followed by workspace all-target Clippy. Evidence is retained under
`output/diagnostics/gghh-homogeneity-1`.

The actual seven-parameter ggHH input then completed the public
`ParametricIntegrand::from_family` call with all 179 terms in 1.881 seconds.
The whole diagnostic process exited 0 in 3.719 seconds with a peak resident size
of 70,816 KiB. This establishes parameterization and admission, not geometry,
artifact publication, or numerical integration. The earlier bounded attempts
and incorrect diagnostic control expectation are retained separately; the final
exact-field controls do not rely on the default statistical zero test.
