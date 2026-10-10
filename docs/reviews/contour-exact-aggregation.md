# Exact-offset aggregation and numerical coefficient normalization

The direct native vector evaluator receives already aggregated mathematical
Atoms. It does not perform symbolic cancellation while evaluating them. A
control exposed the distinction: native construction of `left = f(p) + 1/p`
and `right = -left` leaves their sum as `f(p) - (f(p) + 1/p) + 1/p`.
Although mathematically zero, that Atom still executes `f(p)` and the singular
division at `p = 0` unless aggregation distributes the numerical minus sign.
This was a real aggregation gap, not an error in Symbolica's new shared-cache
evaluation API.

The shared `generation::normalize_exact_coefficient` helper uses only native
`AtomCore::expand_num`. Ordinary generation applies it when finalizing exact
Laurent coefficients. Selected-record and resident assembly apply the same
operation after summing the saved exact contributions and before pruning or
merging exact-root associations. Consequently, combining saved records on
loading performs this narrow symbolic normalization too; it does not regenerate
the smooth density, construct an evaluator, or repeat Horner/CPE optimization.
No new operation is inserted into the sample-evaluation path.

## Native reuse evidence

This operation's reuse evidence was collected on public Symbolica consumer
`1e1cb169bec35ed3b8536050f789321f063a2047`. The later `516beb37` selection retains
the same normalization API; the [current milestone audit](contour-dynamic-milestone-audit.md)
records its complete native rerun.

- Public API: `AtomCore::expand_num` documents distribution of numerical
  coefficients, for example `2*(x+y)` to `2*x+2*y`.
- Source and owner tests: `src/expand.rs::expand_num_impl` treats
  `AtomView::Fun` as opaque, distributes coefficients through arithmetic nodes,
  and retains general factored products. Native tests cover sparse and
  collapsed children and idempotence.
- Focused native probe:
  `target/contour-exact-aggregate-normalization.rs` verifies that `expand_num`
  and `together` both cancel the problematic sum, while `cancel` alone does
  not. The narrower `expand_num` leaves `f(2*(1+p))` unchanged and produces
  `[0,6]` at `p=0` without executing the deliberately failing cancelled
  callback. The probe uses the existing native vector evaluator.

The maintained exact regression calls the same production helper and first
asserts that the raw sum is not a literal zero. A separate regression retains
an actual tagged dynamic strength with factored coefficient and safety
arguments and verifies that its complete Atom identity survives normalization.
It also verifies that `(p+1)*(q+1)` remains factored. These checks protect the
saved root-to-certificate association rather than assuming that any CAS
simplification preserves callback arguments.

The focused standalone probe passed before the production helper was added.
The current native test binary then passed all ten exact controls, all seventeen
recipe controls, and all forty-five artifact controls (the latter two each
exclude one guarded subprocess entry). Artifact coverage includes both live
resident assembly and restored saved records with the deliberately factored
numeric sign and a singular per-record value at the binding point. Both paths
produce the exact complete vector `[0,6]` and the same native content identity.
These focused gates do not imply completion of the broader dynamic-contour
scientific acceptance matrix.
