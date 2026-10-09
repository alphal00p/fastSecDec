# Certified real square-root owner audit

2026-10-09. The sign-aware dynamic envelope needs square roots of finite
nonnegative real balls. This audit concerns that owner primitive, not a claim
that inherited complex or transcendental ball operations are certified.

## Reuse evidence and counterexample

The public Numerica `RealBall` documentation explicitly excludes square root
from its certified operations. Source inspection confirms `RealBall::sqrt`
calls `monotone_increasing(Float::sqrt)`, while `Float::sqrt` returns a rounded
point approximation. Native public directed operations include addition,
subtraction, multiplication and division; no public `sqrt_round` or alternative
certified real square-root enclosure was found in the API/source/tests.

A focused executable using the full tested owner library at public consumer
revision `eccd0396599053f245e91b86fd73720a3b07e88c` constructs the exact input 2
at eight-bit precision. Its square root is the singleton `[181/128, 181/128]`.
The exact rational square `(181/128)^2` is below 2, so this singleton cannot
enclose the mathematical root. This reproduces the documented limitation
without relying on a higher-precision numerical oracle.

## Narrow owner construction

The isolated Numerica change keeps the existing native floating square root as
an initial positive finite guess. For an exact nonnegative endpoint `a` and any
positive finite guess `g`, the true root lies between `g` and `a/g`. Therefore

```text
lower = min(g, directed_down(a/g))
upper = max(g, directed_up(a/g))
```

enclose it independently of the guess's accuracy. Apply the lower construction
to the ball's lower endpoint and the upper construction to its upper endpoint,
then use the existing outward midpoint/radius constructor. Zero endpoints are
handled exactly. Negative or nonfinite represented domains and unusable guesses
produce the existing invalid-ball diagnostic; the implementation never clips a
negative lower bound to zero.

This is a small owner-library numerical improvement. It adds no FastSecDec
square-root arithmetic, CAS, generic interval package or root solver, and changes
no scalar comparison, zero predicate, complex branch or evaluator instruction
semantics. `ComplexBall::sqrt` remains outside the certified contract.

## Verification

An independent runtime-agent review verified the endpoint proof and source.
The new tests compare exact rational squares against both the intended rational
input and the entire represented input interval after constructor widening.
They cover irrational singleton roots, exact zero and perfect squares, positive
intervals, zero endpoints, binary scales `2^±2048`, multiple precisions, a
rounding sweep, and negative/NaN/infinite domains.

The focused owner tests pass **6/6** with native GMP/MPFR and **6/6** with the
portable Malachite/Astro backend. The existing interval tests pass **10/10**
with each backend.

The independently based owner change is published as
[Symbolica PR 58](https://github.com/symbolica-dev/symbolica/pull/58), revision
`59c5eaa04870c94c5c6aa79f52e9960169a01d58`, authored and published by
ValentinHirschi. GitHub denied formal reviewer assignment, so an explicit
[`@benruijl` review invitation](https://github.com/symbolica-dev/symbolica/pull/58#issuecomment-6088599198)
is posted. The standalone patch is retained at
[`numerica-certified-real-sqrt.patch`](../dependency-patches/numerica-certified-real-sqrt.patch).
The coordinating agent subsequently included it with the restored existing
PR 54 prerequisite in public consumer revision
`7ec1be45ef92ae3b154e0d4ce754c0bdf3d9d0ca`. Native and portable six-test square-root
controls also pass on that combined head; consuming FastSecDec checks are
tracked separately.
