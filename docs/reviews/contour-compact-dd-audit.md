# Compact-map double-double endpoint audit

2026-10-10. Independent review of the failed raw double-double comparison in
`contour::definitions::tests::compact_radius_maps_preserve_eager_jit_precision_and_native_jets`.
The failure was a nonfinite endpoint evaluation in both representations, not
loss of compact-versus-materialized numerical agreement.

The isolated native probe reuses the actual definition factory, native
`FunctionMap`, original radius callback/helper, native symbolic derivatives and
native evaluators. It checks the image, its first derivative and both selected
second derivatives at `x = 0, 0.03125, 0.3, 0.8, 1`, with
`y = 0.4, S = 0.8, L = 0.2, R = 1`. Every materialized compact expression is
exactly native-equal to its original expression before compilation.

At the first four points, the double-double vectors differ by at most
`3.0814879110195774e-33`; the 192-bit vectors differ by at most
`3.982729777831131e-59`. At `x = 1`, both double-double evaluators report the same
coefficient admission error and return NaNs. Both 192-bit evaluators succeed
and agree on all four complex outputs, including the nonzero derivatives.

To distinguish this from a hypothetical owner arithmetic defect, a private copy
of the callback was instrumented at its existing admission-error return. Only
the error string changed. In **both actual combined programs**, coefficient
index zero reaches the callback as
`a2 = 1 - 3.0814879110195774e-33`, measured by native double-double subtraction.
Its rounded f64 centre is `1`. The exact expression proves `a2 >= 1`, with equality
at this endpoint. A separate coefficient-only native evaluator produces exactly
`1`; its different arithmetic ordering does not establish the input of the
combined image/derivative program. The instrumented callback does.

The public `ExpressionEvaluator::map_coeff` selects the target domain's fixed
precision; `DoubleFloat` uses its native compensated arithmetic. The contour
callback's strict coefficient-domain check therefore correctly refuses this
rounded value. No new algebra, evaluation implementation, clamping or relaxed
finite-value tolerance is justified. The maintained test should distinguish
successful finite parity from matched explicit domain refusal, then verify the
same complete vector at higher precision. This raw-evaluator probe alone does
not claim that a production precision policy's retry path was exercised.

Evidence is kept out of Git under `target/contour-compact-dd-audit/`: `probe.rs`,
`probe.log`, original/compact native IR dumps, and the diagnostic-only copied
callback modules. The native owner is the existing public Symbolica/Numerica
`516beb37` selection. No owner-source change or upstream PR is indicated by this
ordinary finite-precision rounding case.
