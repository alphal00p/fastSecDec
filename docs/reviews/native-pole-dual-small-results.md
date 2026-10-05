# Exact native pole extraction and dual controls

This disconnected diagnostic follows the source research in
`native-original-oracle-numeric-series-audit.md` and its independent review.
It reads no named-candidate output or actual captured density. Production
generation and the failed original-expression oracle attempts remain unchanged.

`output/probes/native_pole_dual_small.rs` constructs small original densities
from analytically admitted regular factors and explicit affine endpoint
denominators. The original source description bounds each branch by six poles;
it does not infer this bound from the final expression or candidate results.
Native `collect_factors` and multiplication by epsilon to the sixth power
perform the exact transformation. Native monomial reversal is checked without
expanding residuals. A narrow exact operation-domain check rejects intermediate
zero inverses, nonpositive bases for noninteger or regulated powers/logarithms,
nonpositive Gamma centers and unknown variables/functions.

The ordinary exact native evaluator is vectorized by `Dualizer` with
scalar-first `HyperDual` component shape, then independently mapped to native
512-bit and 1024-bit MPFR evaluation. No caller factorial, convolution,
derivative, interpolation or Laurent arithmetic is supplied. Native Series of
the unchanged original density provides the complete signed-order reference,
with its own strict remainder bound. Native dual shape is an analyticity/Taylor
coverage statement, not a `Series::absolute_order` certificate.

Six cases cover known polynomial normalization, regular Gamma with positive
regulated powers, mixed logarithmic/complex coefficients, exact leading
cancellation, a tiny nonzero leading coefficient and a negative requested
maximum. Every order from minus six to the requested maximum is retained.
The tiny leading value additionally requires nonzero native values and relative
agreement without an absolute unit floor. Complex coverage requires a nonzero
imaginary reference coefficient. Nine typed rejection controls and a separate
structurally exact-zero control preserve nonvacuity of the six numeric vectors.

HEP and Numerica reviewed the source, including the added tiny-value guards.
The initial standalone compilation passed against the same fixed native rlib
as the independent original oracle; source/build archives are retained in
`output/diagnostics/native-pole-dual-small-build-1`. The reviewed runner allows
one new small-control process, with 180 seconds plus five seconds grace,
30 GiB address space and CPU8. The concrete attempt is frozen under
`output/diagnostics/native-pole-dual-small-1`.

The bounded process exited zero without timeout in 7.721823169 seconds, with
9,216 KiB peak child RSS, 7.648750 seconds user CPU and 0.029808 seconds system
CPU. All 19 immutable files passed postflight. These are correctness diagnostic
times, not a controlled performance comparison.

All six cases passed, retaining 40 signed coefficient rows: five complete
vectors minus six through zero and one minus six through minus two. Every row
passed dual precision agreement, independent original-Series precision
agreement and the cross-route comparison separately at both precisions. The
original Series remainder was one for finite-order cases and minus one for the
negative-maximum case. The candidate's coverage remains the independently
bounded analytic Taylor shape; no Series bound is attributed to it.

Known polynomial normalization, exact leading cancellation, relative-only tiny
nonzero checks and nonvacuous complex coverage passed. All seven intermediate
domain rejection cases, two source-bound/denominator rejection cases and the
separate exact-zero input control passed. Native Gamma/polygamma at the regular
center and the seventh Taylor component executed using native MPFR support.

The frozen source SHA-256 is
`386ce084e431154e0281eac4d1e7ecf51c0049dd94c5478001bf30b42f2db2c4`;
binary SHA-256 is
`392ed5e30ab652b964f36ffb12eeab02154a4ada89beabac91e56a974317560d`.
HEP independently rechecked all 19 frozen files, all six cases and 40 rows,
the polynomial/tiny/complex guards and all rejection/zero controls;
`independent-review.json` records acceptance of this small gate. This result
does not apply the transformation to the actual captured expression or certify its coefficients.
Any actual oracle needs its own exact source/pole/domain admission and a
separate coverage format; the old Series-only reader must not receive an
invented `native_absolute_order` for dual components.
