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

## First complete native double-box integration

The preserved Taylor artifact `81d8f31509947ae4dcb7dd55b8066630eefc6961e0a5f9ac05b21cc7921ec2c9` was integrated over all 102 representative sectors with 1024 lattice points per shift, eight complete independent shifts, seed 18931 and Korobov3 periodization. The already-weighted callback and native whole-vector replay were used. All 835,584 samples completed with zero evaluation failures; 299,215 samples required native rescue, with a maximum of 448 bits. There were 181 additional first/growth replays. Artifact loading took 17.443 s and serial diagnostic integration took 164.505 s. This was a development-build diagnostic, not a matched performance acceptance measurement.

| Laurent order | Native estimate | Estimated standard error |
| --- | ---: | ---: |
| −4 | 0.00504150447 | 0.00141677163 |
| −3 | 1.49731323279 | 0.01383094434 |
| −2 | 1.20170491109 | 0.02692830320 |
| −1 | 2.87390202263 | 0.06227307302 |
| 0 | −15.48864782850 | 0.34970095575 |

The complete covariance, settings, precision counts and historical-target differences are retained in `output/probes/double-box-integral-taylor-1024x8.json`; the runtime log is adjacent. The historical target came from a manual numeric result and has no certified uncertainty. Its leading zero differs from this estimate by about 3.56 of the reported native standard errors. This requires an independent pole/convergence investigation; neither the filename nor this small-shift diagnostic establishes a certified discrepancy or successful scientific parity. No code or target was adjusted to force agreement.

## Independent leading-pole proof

The complete generated coefficient of epsilon to power minus four has only ten nonzero sector contributions. Their sum depends on two coordinates, `x` and `y`, and is exactly the `x` derivative of

```text
9*x^2 * ((1 + (1+y)*x)^(-4) - (x + 1+y)^(-4)).
```

This primitive vanishes at both `x=0` and `x=1` for every remaining coordinate. The ignored regression `double_box_leading_pole_is_an_exact_total_derivative` reads the actual preserved artifact, verifies the derivative identity and both boundaries with native Symbolica, and independently obtains a rational primitive through native `RationalPolynomial::integrate`. It passed in 0.28 s; evidence is in `output/double-box-leading-pole-proof.log`. Thus the exact leading coefficient is zero independently of the historical target. The numerical output remains unchanged and is assessed against this identity in subsequent convergence checks.

## Independent-seed 64-shift convergence diagnostic

The same preserved artifact and precision policy were evaluated with 1024 points per shift, 64 complete shifts, independent seed 18932, and four caller-owned numeric workers. Fixed sector ownership preserves increasing shift/point order within each sector independently of worker count. All 6,684,672 evaluations completed, with zero failures, 2,394,073 native rescues, 156 additional weighted replays and a maximum of 512 bits. Loading took 18.634 s and integration 427.168 s. This development run overlapped unrelated compilation and is not a matched performance acceptance measurement.

| Laurent order | Native estimate | Estimated standard error |
| --- | ---: | ---: |
| −4 | −0.000181020268 | 0.000588690140 |
| −3 | 1.50134495037 | 0.00436800280 |
| −2 | 1.26697148671 | 0.0120945978 |
| −1 | 3.03985178803 | 0.0434962853 |
| 0 | −14.5775888121 | 0.240008021 |

The leading coefficient is −0.3075 estimated standard errors from its independently proven exact zero. This removes the earlier small-shift discrepancy as evidence of a systematic leading-pole error; it does not prove the uncertainty model for arbitrary inputs. Higher coefficients remain diagnostic comparisons because the historical target has no certified uncertainty. No coefficient was replaced by the analytic zero or adjusted toward that target.

`output/probes/double-box-integral-taylor-1024x64-seed18932-workers4.json` retains the complete 5×5 covariance, all 64 native common-shift totals and IDs, worker ordering, settings, precision counts and historical target provenance. The adjacent `.log` records progress. The estimate and covariance come from the existing native QMC reduction; the raw totals are retained for independent inspection rather than used to construct another estimator.
