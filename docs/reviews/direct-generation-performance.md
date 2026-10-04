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
of this new fallback is measured below. Accuracy policy was not loosened.

The saved 152-sector artifact isolates precision evaluation from generation and
compilation. On the fixed Kuo-1024/shift-481/Korobov-3 diagnostic points (64 per
sector), native numeric rescue evaluated 9,728 vectors in **24.729 seconds**:
3,289 received conditioning checks and 2,465 required MPFR. The full 48,640
coefficient values and chosen precision counts are retained in
`output/probes/double-box-precision-numeric-uncached.json`.

The first worker-local cache experiment rounded starting precision upward to
binary tiers and retained at most four native evaluator/buffer sets per worker
sector. It returned **bitwise identical coefficient values**, but regressed to
**38.533 seconds**. Binary rounding moved 1,184 rescues to 512 bits and another
44 to 1,024 bits; the original calculation often needed only 258–498 bits.
The cache therefore did not establish a performance improvement. The next
controlled experiment keeps the same bounded native cache but rounds starting
precision in 32-bit steps, preserving the lost-bit lower bound and the required
agreement between two distinct increasing precisions. It completed in
**19.601 seconds**, again with identical check/rescue counts and all 48,640
coefficient values bitwise unchanged: approximately 1.26 times the throughput
of the uncached numeric evaluator on these diagnostic points. Artifact loading
and compilation took 34.246 seconds and are excluded from this measurement.
The report is `output/probes/double-box-precision-numeric-cached32.json`.
Cache memory is bounded per sector; the aggregate budget for thousands of
sectors and several workers still needs measurement.

Reference precision dispatch also explains an important performance difference:
its 32-decimal-digit lane uses Symbolica's native `DoubleFloat` evaluator, not
MPFR. In the reference Python binding, `python_api/evaluator.rs` around line
1237 calls `evaluate_double_float_complex`; the higher precision path around
line 1244 caches a native Float evaluator for the selected bit count. Reference
boundary checks inspect singular axes. The initial Rust metadata has only a
single conservative cancellation degree and checks every coordinate. Planned
per-piece, per-axis metadata must preserve the actual lost-bit guard and full
vector cancellation while avoiding checks triggered by unrelated coordinates.

The next implementation slice carries the actual cancellation orders of each
retained Taylor remainder. A point's conservative lost-bit estimate is the
maximum, over these remainder terms, of the sum of
`degree[axis] * log2(1 / coordinate[axis])`. This also controls the conditioning
trigger: a high Taylor degree can require checking an ordinary interior point,
while a tiny coordinate unrelated to any subtraction should not trigger it.
The raw per-term data and derived degree are stored in artifact identity;
historical artifacts without those rows retain their old conservative total
degree bound. All precision thresholds and the two-precision agreement policy
remain unchanged.

Common parameter-independent prefactors are detached before differentiation and
reattached around the complete sum of endpoint pieces. Terms with identical
endpoint powers but different prefactors are first recombined, so factoring
does not remove a cancellation that makes their sum integrable. Complete mapped
densities, including these prefactors and every numerator, are passed to native
graph canonization and exact permutation verification before representatives
are integrated with their multiplicities.

The library also exposes explicit Taylor and integration-by-parts strategies.
For a regulated power `lambda = a + b*eps` with `a <= -2`, the latter repeatedly
uses the exact meromorphic identity
`I(lambda,f) = f(1)/(lambda+1) - I(lambda+1,f')/(lambda+1)`
until a single Taylor subtraction is sufficient. Native Symbolica owns the
derivatives, substitutions and Laurent series. The boundary at one and its
epsilon-dependent denominator remain in the same correlated output vector.
The default remains Taylor while independent integral identities and the
representative workload measurements validate the new strategy. Pointwise
Taylor formulas must not be compared to an IBP integrand: their integrals agree,
but their finite integrands need not agree at individual sample points.

The paired fresh double-box probes use the same input, precision policy and
64-point-per-representative rule. Complete-density symmetry reduces the 152
charts to 102 verified representatives in both strategies:

| Strategy | Generation | O2 compilation | Portable artifact | 6,528 evaluations | MPFR rescues |
| --- | ---: | ---: | ---: | ---: | ---: |
| Factored Taylor | 6.849 s | 6.527 s | 33.13 MB | 1.667 s | 2,302 |
| Factored IBP | 11.107 s | 11.421 s | 59.04 MB | 1.548 s | 1,309 |

Taylor therefore remains the default. IBP lowers the frequency of rescue but
enlarges the expressions and generation work on this example. The independent
strategy tests compare complete integral identities, including higher endpoint
powers, rather than individual Taylor/IBP sample values. These diagnostics do
not establish full double-box accuracy or matched reference performance.
Artifacts and stage reports are retained as
`output/probes/double-box-{taylor,ibp}-factored.fsd` and
`double-box-generation-{taylor,ibp}-factored.json`.

After these paired measurements, an exact cleanup removes the derivative taken
after the last requested Taylor coefficient; that derivative was never used.
The next whole-integral measurements will include this cleanup.

The mapped-factor cleanup preserves the primary factored Atom whenever no
coordinate monomial needs removal. In nonnegative charts, nonvanishing
coordinate faces provide a fast path that avoids enumerating the support of
regular weights such as `(1+x)^10000` and `(x+y)^10000`; those weights remain
compact through Laurent coefficient generation. When stripping is necessary,
native signed polynomial collection followed by `mul_exp` handles ordinary and
orthant-infinity charts, with checked exponent bounds. Singular residuals still
receive exact native coefficient/constant-term and boundary validation.

A factored Taylor identity need not normalize to literal zero. The final
required derivative provides a sufficient exactness test: if it is independent
of the current axis, the Taylor polynomial exhausts that regular function and
its remainder is zero. This eliminates spurious numerical pieces without an
expansion or an unused additional derivative. The existing three-axis
polynomial identity exercises intersecting faces and negative regulator slopes;
the compact-power regression guards against reintroducing dense primary
expressions during mapping or Laurent cleanup.
