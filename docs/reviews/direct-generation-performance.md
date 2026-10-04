# Direct generation performance autopsy

The first native massless double-box probe produced 152 sectors and orders
epsilon^-4 through epsilon^0 in 100.309 seconds. Geometry took about 0.1 seconds;
the dominant stage was the Laurent expansion of already substituted and
subtracted sector expressions. Applying global `together()` made this worse:
the second sector exceeded two minutes and the process reached about 2.5 GB.
That experiment was stopped and global rational combination was removed.
These are diagnostic observations on the development build, not a matched
performance-parity claim.

The reference direct path in
`FastSecDecPathFinder/src/integrand.py` is materially different:

- `build_explicit_sector_formula` (around line 7810) produces direct substituted
  output kernels by composing an endpoint assembler with regular coefficient
  expressions. The single dual U/F evaluation path is not used in this design.
- `_two_stage_assembler_expressions` (around line 6845) obtains reusable,
  topology-independent endpoint expressions whose regular coefficients remain
  abstract symbols until the last substitution.
- `_g_coefficients_by_symbolic_diff` (around line 6762) forms each regular
  epsilon coefficient from a factored base times powers of
  `monomial_log + b_U log(U) + b_F log(F)`, including numerator coefficients.
  It differentiates only the requested Taylor multi-indices.
- `_two_stage_derivative_fused_components` (around line 6963) groups requests by
  boundary, caches the corresponding regular Taylor data, and substitutes the
  actual residual polynomials late. `_explicit_finite_sector_outputs` (around
  line 7610) uses the same factored log-power structure for finite sectors.
- `src/subtraction_formula.py::_EndpointProjectorContext::build_outputs`
  (around line 2276) runs Symbolica series on a small formal template, not on
  dense substituted polynomials. `_ibp_terms` (around line 2457) optionally
  lowers higher-power endpoints to logarithmic ones, retaining the boundary
  terms at one and the exact epsilon-dependent denominators.

The immediate algorithmic correction is late substitution: keep
epsilon-independent coordinate expressions opaque while native Symbolica
expands the small epsilon template, then substitute the actual expressions
into complete Laurent outputs. Native `replace_map` is top-down and stops
descending after a replacement (`symbolica/src/id.rs:1668`); this supports
maximal opaque expressions without a second algebra implementation. Cache
identical templates within generation. Symbolica continues to own all series,
derivatives, substitutions and arithmetic, including prefactor-pole order
propagation. Further factor-preserving regular-coefficient and IBP planning is
to follow if the measured template correction does not remove the bottleneck.

Correctness gates include fractional endpoint exponents, Gamma-prefactor
poles, complete correlated Laurent vectors, higher Taylor subtraction,
analytic bubble/triangle identities, and a boundary-geometry regression.
An assertion of no interior threshold does not certify that boundary zeros
have been resolved. Mixed-sign factors must also pass exact boundary checks;
unsupported affine boundary geometry is rejected until appropriate charts
exist. The proof is also applied to actual mapped residuals: positivity of
original faces alone does not exclude zeros of a leading form along a finite
ratio after a blowup. An independent tangential-origin regression covers this.

The first correction preserves Taylor remainders as factored differences,
abstracts maximal epsilon-independent coordinate expressions before the native
series call, caches identical templates, and restores the expressions afterward.
All 16 generation tests and all five independent mathematical tests passed after
this change, including the production Gamma/polygamma conditioning regression.

The corrected double-box probe generated the same 152 sectors and five Laurent
orders in **26.211 seconds**, followed by **12.763 seconds** of portable O2
kernel construction. This is about a 3.8-fold reduction in diagnostic generation
time; it is still not a matched reference benchmark. Expression artifacts are
retained under ignored `output/probes/double-box-generated.fsd` and
`double-box-compiled.fsd`.

The initial precision measurement used the older Atom-based fallback and was
stopped after 3,264 points: 1,173 required conditioning checks and 970 used MPFR,
taking 70.1 seconds. Thus rescue is not rare for this workload. The production
fallback now uses previously built native numeric evaluator IR, with exact
constants remapped to Float precision and no Atom/Workspace operations on
workers. The short mathematical and complex-worker regressions pass; throughput
of this new fallback remains to be measured. Accuracy policy was not loosened.
