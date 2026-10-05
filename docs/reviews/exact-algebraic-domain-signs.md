# Exact algebraic coefficients in no-threshold admission

The ordinary native ggHH CLI now completes input contraction and seven-parameter,
179-term Gaussian parameterization. Attempt 4 stops at domain admission because
the existing coefficient-sign check accepts only rational constants. Its actual
F polynomial includes coefficients containing `sqrt(11)`; no physical point or
threshold assertion was changed.

A bounded read-only probe parsed the exact retained F expression, collected its
coefficients with Symbolica's fixed-variable polynomial conversion and exact
coefficient zero policy, then used `AlgebraicContext::from_atom`, `convert_atom`
and `RealEmbedding::try_sign`. All 57 monomial coefficients are positive. There
are five distinct coefficients, no negative or zero coefficients, and no unknown
signs. Sign evaluation took 0.00128 seconds; the process exited 0 in 0.02836
seconds with all input/binary hashes unchanged. Evidence is retained in
`output/diagnostics/gghh-algebraic-sign-1`.

The native owner already supplies the needed operation. `AlgebraicContext`
records embeddings for radicals and converts expressions into the resulting
number field. `RealEmbedding::try_sign` uses native certified real-root/sign
machinery and rejects a non-real image. Existing Symbolica tests cover positive
and negative embedded roots and non-real rejection. No approximate numerical
positivity test, custom radical comparison, or alternate symbolic engine is
needed.

The small private `generation/domain/sign.rs` adapter retains the rational fast
path and caches repeated exact coefficient outcomes within an admission call.
Unsupported or non-real expressions return an inconclusive result. Initial
coefficients, exact interior witnesses, corner zeros, asserted faces and resolved
residuals use this same owner. The existing branch, boundary and explicit
assertion rules remain in place; sampled positivity is never a certificate.
Coefficient gathering and residual constant extraction disable the native
AtomField statistical zero test, so small algebraic coefficients reach exact
sign admission instead of being discarded before certification.

Independent source review accepted reuse of the native owner and preservation
of the admission rules. Root review additionally requested the exact-field
gathering above; the final source and a tiny negative coefficient regression
include that refinement.

Four focused controls passed against the unified native HEPKit owner for positive and negative algebraic
coefficients, fractional-power branch rejection, unsupported/non-real constants,
exact algebraic zeros, a tiny negative coefficient and unchanged witness/face
rules. The process exited 0 in 0.07285 seconds; all bound binary and source
hashes remained unchanged. Existing generation (16), generation-context (8),
regression-gap (4), native scalar-master (5) and numerator-reduction (5) controls
also passed against the unified owner: **42 passed, zero failed or ignored**
including the four domain controls. The five reduction tests include rank five
and a Gram-degenerate box. All six processes exited 0 and were reaped, with
source and executable postchecks passing. These are regression observations;
the CLI build overlapped the reduction tests and their times are not benchmarks.
Evidence is retained under
`output/diagnostics/algebraic-domain-1`. The original
ordinary-CLI failure is retained, and no artifact or numerical result is claimed.

Ordinary CLI attempt 6 is pending with the unchanged ggHH card and a
180-second deadline plus five-second cleanup grace, under a 30 GiB address-space
limit. Build 7 uses the unified dependency owner. Attempt 5 did not launch the
CLI: a diagnostic library selector expected Cargo target kind `lib`, while
SymJIT declares `rlib`. That launcher-only failure is preserved separately; the
corrected exact-name/unique-rlib selector reuses the same successful build.
