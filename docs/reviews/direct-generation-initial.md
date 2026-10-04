# Independent direct generation review

Date: 2026-10-04. Reviewer: the Numerica/runtime implementation agent, separately
from the generation author. Scope: mapped integrands, endpoint subtraction,
Laurent vectors, and no-threshold boundary certification. This review does not
establish Pathfinder performance parity or validate all scientific fixtures.

## Findings and resolution

**Resolved correctness finding: a no-threshold assertion cannot certify boundary
resolution.** On the unit square, `P=(x-1/2)^2+y` is strictly positive inside the
domain and at every corner. Nevertheless, `P^(-3/2+eps)` has Laurent residue
`2/eps`. Integrating over y first gives

```
[ integral_0^1 ((x-1/2)^2+1)^(eps-1/2) dx - 2^(-2eps)/eps ] / (eps-1/2).
```

An origin-only Newton fan misses the affine boundary zero at `(1/2,0)` and could
previously produce a vector with no pole. Generation now conservatively rejects
uncertified boundary geometry even with `assume_no_threshold=true`. No affine
charts or second-phase contour machinery are implied by this rejection.

The author additionally found that original-domain faces alone are insufficient:
`P=(x-y)^2+2*x^2*y+2*x*y^2+x^3+y^3` has positive coordinate faces and is strictly
positive in the open square, but the substitution `x=t*y` leaves `(t-1)^2` on
the exceptional face. The final check therefore examines the **actual mapped
residuals**. Mixed residuals need a certificate on every cube face: coefficients
of one strict sign and a nonzero constant. Each certificate bounds that entire
closed face away from zero, including its intersections. Uniform-sign residuals
are safe when their constant term is nonzero, the simultaneous-valuation
invariant of the Newton fan. Uncertified mixed orthant/projective boundary
geometry remains explicitly unsupported. Rational interior samples are used
only to disprove an assertion, never to prove positivity.

## Mathematical checks

- Monomial substitution uses signed exact exponent matrices. The Jacobian and
  input monomial powers are added before extracting individual factor
  valuations; numerator factors share the same map without refining the fan.
- For an endpoint exponent `a+b*eps`, the subtraction removes `floor(-a)`
  Taylor terms when `a <= -1`. Repeated native differentiation and division by
  the degree generate the factorials correctly. The integrated boundary term
  has denominator `a+k+1+b*eps`. A nonzero unregulated subtraction term produces
  a typed error rather than an invented dimensional pole.
- Boundary terms retain the original sector's integration coordinates. Their
  integrated axis contributes volume one, so the whole Laurent vector remains
  correlated at each sample. Only a whole coordinate-independent vector is
  moved into the exact contribution.
- The prefactor, subtraction denominators, and density enter native Symbolica
  series together. This accounts for prefactor poles requiring extra orders.
  Integer Laurent orders and absence of a surviving regulator are checked.
- Projective input validates total homogeneity including all powers and measure.
  The independent geometry review separately checked normalized projective
  measures and signed positive-orthant maps.

## Expression representation

The user's factored-expression requirement is essential to this path. Native
Symbolica expressions remain the symbolic source of truth. The current Taylor
remainder preserves products and powers; the Laurent pass replaces
regulator-independent coordinate subexpressions with opaque placeholders before
native series expansion and restores them afterward. It does not split numerical
Laurent terms into separate integrals.

Sparse polynomial coefficient access is necessary for Newton support and exact
sign certificates. Such diagnostic conversion must not force the primary
coefficient expression into an expanded representation. At review time the
mapping layer still explicitly expands each residual polynomial after removing
its monomial, rather than preserving a separate factored residual. This is a
representation/performance follow-up for the generation author; whole-density
or whole-coefficient expansion is not justified by those certificates. The
complex kernel adapter uses native complex evaluators and requires no symbolic
real/imaginary expansion.

## Independent regression evidence

`crates/fastsecdec/tests/independent_generation.rs` contains independent checks:

1. The meromorphic integral of `x^(-3/2+eps)` equals
   `-2 - 4*eps - 8*eps^2`, including the sign.
2. The complete generated vector for
   `eps*Gamma(eps)*integral_0^1 x^(eps-1)/(1+x)^2 dx` agrees with independently
   derived coefficients through order one. A seeded 4096-point, 16-shift QMC
   integration checks means against six estimated standard errors plus `5e-10`.
3. The affine boundary-pole example above is rejected.
4. Zero original faces and uncertified mixed orthant/projective boundary
   geometry are rejected.
5. The tangential-origin example is rejected after inspecting mapped residuals.

The first three tests passed in the reviewer's own run under Rust 1.98.1/Linux.
The author then ran `cargo test -p fastsecdec --test generation --test
independent_generation --locked -- --test-threads=1`: all 16 author tests and
all five independent tests passed after the mapped-residual fix. The final
command was shared to avoid competing Cargo builds in the common workspace.
This audit is not a benchmark and
does not certify the numerical precision heuristics separately implemented in
the kernel layer.
