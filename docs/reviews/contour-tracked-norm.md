# Native tracked hypot in Numerica

2026-10-09. Narrow owner correction supporting the smooth sign-aware envelope.
This review concerns native numeric reuse, not a new contour certificate.

## Public API, source and executable evidence

Numerica 3.0.1 already provides stable `Real::hypot` for its scalar domains.
`ErrorPropagatingFloat` inherits a generic fallback because it does not expose
scalar ordering. The result unnecessarily squares its inputs before taking a
square root. A primitive-only probe of `hypot(1e200, 1e-3)` returns a finite
ordinary-f64 result but a nonfinite tracked result. The same issue affects
`Complex<ErrorPropagatingFloat>::norm`. No alternative tracking-preserving scaled
primitive was found in the public API or source.

The [owner patch](../dependency-patches/numerica-tracked-hypot.patch) adds only
an `ErrorPropagatingFloat::hypot` override. It calls the existing native centre
primitive, forms bounded derivative weights `|x/r|` and `|y/r|` in the original
numeric domain, and combines the existing absolute errors according to the
owner's local linear propagation convention. It retains previous arithmetic at
the nondifferentiable origin and for nonfinite results. No ordering, zero
predicates or complex branch-selection rules change.

This remains ordinary numerical error tracking. Neither the weights nor a finite
tracked error constitute a certified ball enclosure.

## Rejected broader candidate

The earlier unconsumed local commit `be9dc71774d97c4f48eb0be92bdf3281a33b20ff`
forwarded scalar ordering and range guards. Independent review rejected that
approach before adoption or publication. It activates existing complex axis
shortcuts that mistake an uncertain zero centre for an exactly known zero.
Concrete counterexamples include:

- `sqrt(1e-6 + i*(0 +/- 1e-12))`: imaginary uncertainty fell from approximately
  `5e-10` to `1e-12`.
- `1/(1e-200 + i*(0 +/- 1e-220))`: an imaginary uncertainty of order `1e180`
  was replaced with zero.
- An uncertain zero numerator divided by a small complex denominator lost the
  corresponding uncertainty amplification.

Changing the global exact-zero predicate also affects evaluator conditional
branch selection, so that wider change is not justified by the contour task.
The accepted narrow override preserves the square-root uncertainty and the
previous nonfinite diagnostics for the extreme complex divisions. It does not
pretend to resolve those existing division limitations.

## Isolation and validation

The accepted local combined owner commit is
`5fc38caaaa9ebca9158d8c5977bf57073737950a`, above the ball-domain and prepared-root
changes. Author and committer are `ValentinHirschi <valentin.hirschi@gmail.com>`.
Its separate publication branch is based directly on upstream Symbolica `main`,
with an identical Numerica subtree and no dependency on either other PR.
It is published as [Symbolica PR 57](https://github.com/symbolica-dev/symbolica/pull/57),
authored by ValentinHirschi. GitHub denied formal reviewer assignment, so an
explicit `@benruijl` review request is posted on the PR.

Full native GMP/MPFR and portable Malachite/Astro owner libraries compile.
Focused tests pass **6/6** on each host backend, covering f64, double-double and
192-bit Float; both signs and argument orders; scales `1e200` and `1e-200`;
uncertainty retention; zero centres; NaNs/infinities; unchanged scalar predicates;
and complex square-root imaginary uncertainty. Existing native API regressions
pass **14/14**, and complex tests pass **15/15**, including branch cuts, extreme
scales, extended precision and SIMD lane semantics. Portable-host execution is
not a browser/WASM test.

Independent runtime-agent source and executable reviews approve the narrow
replacement. FastSecDec must still distinguish the holomorphic square root in
the spectral callback from a Hermitian complex norm; this correction does not
make those mathematical functions interchangeable. Consuming dependencies and
the separate frozen fixed-contour HEPKit build change only by coordination.
