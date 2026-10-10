# Native higher-dimensional contour determinant audit

This is a read-only design and bounded-capability audit of production
`33030058d2cd0587dd718ef3e7dc9c56ba6952f8`, using its pinned public
Symbolica/Numerica `516beb37d31af8e3d6ee321a7070f407a0b1b42d`. It does not change
the production determinant, extend the retained template cache, or establish
generation or integration performance for an eight- or nine-dimensional chart.

The required `2L6P.a.I` source preparation independently completed with 372
eight-dimensional charts. Preparation is not mapping: its small source records
do not establish that the current direct determinant fallback is affordable.
The required three-loop case needs a separate nine-dimensional check.

## Existing owner paths

FastSecDec's `contour/determinant.rs` uses native `Matrix::det` throughout. Its
cached generic polynomial templates cover dimensions four through six; larger
dimensions pass the physical entries directly to the same native operation.
`AtomField` disables statistical zero testing and enables exact cancellation on
division. That cancellation is necessary for an arbitrary full Jacobian: an
unreduced Bareiss quotient can have a removable zero pivot even though its
determinant is finite.

The current owner source has these public alternatives:

| API | Source behavior and constraint |
| --- | --- |
| `Matrix<F: Ring>::det` | Explicit formulas through dimension three, then native Bareiss with row swaps and exact-field division. |
| `Matrix<F: Field>::det_in_place` | Native Gaussian row reduction followed by the product of diagonal entries. A separate reproduced row-swap sign defect is described below; the proposed bordered route uses `det`, not this method. |
| `Matrix::solve` / `solve_fraction_free` | Native augmented elimination and back substitution; no FastSecDec elimination implementation is needed. |
| `SparseMatrix::det` | Native sparse elimination/LU. Dense contour Jacobians do not acquire useful sparsity merely by using this type. |
| General characteristic polynomial / division-free determinant program | No corresponding public matrix API was found in the pinned source or tests. Specialized low-degree resultant formulas are not a general matrix constructor. |

The dense matrix and `AtomField` sources are unchanged between the earlier
`7ec1be45` inventory and `516beb37`. The previously measured generic expanded
template results therefore remain relevant: dimension seven completed at
324,980,736 bytes peak RSS; dimension eight reached the two-GiB cap without a
result. Those are generic-template measurements, not failures on actual
higher-dimensional physical charts. See
[the original determinant audit](contour-determinant.md).

## A structured native determinant candidate

Write the existing deformation as `z = x - i λ(x) v(x)`, with
`v_i = w_i ∂_i F` and `w_i = x_i(1-x_i)`. Its full Jacobian is

`J = A - i v (∇λ)^T`, where `A = I - i λ Dv`.

This identity retains the existing native implicit derivative of the dynamic
strength. Treating `λ` as constant would give the wrong Jacobian.

For every principal coordinate subset, write
`Dv = D + W H`, where `D = diag(w'_i ∂_i F)`, `W = diag(w_i)` and `H` is the
corresponding Hessian of the real polynomial `F`. At interior cube points,

`W^(-1/2) Dv W^(1/2) = D + W^(1/2) H W^(1/2)`

is real symmetric. Consequently each eigenvalue `β` is real and every
principal determinant of `A` is a product of nonzero factors `1-iλβ`.
For real `λ`, its absolute value is at least one.

At a cube face this conclusion does not rely only on a limiting argument.
Permute the zero entries of `W` first. The corresponding off-diagonal rows of
`Dv` vanish, leaving a block lower triangular matrix with real diagonal entries
in that block and the same positive-`W` symmetric-similarity argument in the
remaining block. This proves the same nonvanishing statement for every
principal subset, including all vertices.

The real premises are actual admission conditions: mapped `F` must be real for
real inputs, physical point parameters are `f64`, and the native dynamic root
requires a finite positive real centre. Complex numerators do not enter `v`.
This proof applies where the existing native strength and its requested
derivatives exist; it does not repair failed causal/root admission or replace
precision rescue. Numerically small elimination pivot ratios can still occur
even though the principal determinants themselves have magnitude at least one.

An existing native-only representation is the bordered matrix

`M = [[A, i v], [(∇λ)^T, 1]]`.

The determinant lemma gives `det M = det J`. Native Bareiss on this ordering
divides only by earlier leading minors of `A`. Those denominators are genuinely
nonzero on the admitted real cube. The final determinant may vanish; no
division by `det J` is introduced. Fixed deformation can use `A` directly.
Thus disabling exact cancellation for this specially constructed matrix has a
mathematical justification that disabling it on arbitrary `J` does not.

This candidate reuses native `Matrix::det`; it introduces neither a determinant
algorithm nor an inverse of the possibly singular full Jacobian. A native
`solve(A,v)` plus determinant-lemma expression is another supported route, but
would duplicate elimination unless an existing owner API shares that work.

## Bounded executable probe

The ignored probe under `target/contour-higher-determinant/` uses the verified
private release dependency graph, with no Cargo rebuild or production source
change. A single monitored run has a 60-second wall limit and one-GiB aggregate
RSS limit. Its intended checks are:

- Native two-dimensional symbolic equality with the full varying-strength
  Jacobian, mixed derivatives through order three, every vertex, and an exactly
  zero full determinant with an invertible `A`.
- Native factored bordered determinants with short independent entries for
  dimensions eight and nine, followed by existing native evaluator lowering.
- Finite interior, mixed-face, all-vertex and zero-strength values against the
  native numerical matrix determinant of the full rank-one update.

The probe passed. The first complete run lasted 0.226 seconds; it finished before
the `/proc` scan sampled its resident memory, so that run's zero sampled peak is
not a memory measurement. An identical 0.228-second repeat added kernel child
resource accounting and reported 28,344,320 bytes peak RSS. Both runs retained
the 60-second / one-GiB monitor and completed without a limit. Concurrent
root-authorized physical work means these are feasibility observations, not an
idle-host performance comparison.

| Physical Jacobian dimension | Bordered matrix dimension | Native factored Atom bytes | Native IR instructions |
| --- | --- | ---: | ---: |
| 8 | 9 | 670,666 | 630 |
| 9 | 10 | 2,763,978 | 876 |

Native construction plus lowering took approximately 0.008 and 0.023 seconds
inside the first process. All eight numerical controls passed, with maximum
absolute component difference `4.22e-15`. The small exact varying-strength,
mixed-third-derivative, vertex and zero-full-Jacobian controls also passed. The
small strength in this control is an explicit polynomial; this is not a new
high-dimensional implicit-root jet acceptance test.

Probe source SHA256:
`6317b56c87ff79ae7a3b3f74c27b799b437a8f3e2fb0403a86a3e6623c87e444`;
binary SHA256:
`b7323d74d2837a19fad1436fd6c00848aa9644d477f808297be18bc7ad01d19e`.
Source, exact link command, stdout and monitor records remain ignored under
`target/contour-higher-determinant/`. The library dependency graph is the verified
private release graph; neither the shared Cargo target nor the user's release
binary was modified.

This passing generic probe leaves actual higher-dimensional chart expression
sizes, full subtraction jets, certified checking, save/restore and complete
generation unverified. Physical entry substitution can substantially enlarge a
small generic expression; native IR sharing does not itself bound the preceding
Atom construction or retained metadata.

## Separate reproduced owner defect

For `[[0,1],[1,0]]`, native `det()` returns `-1`, while `det_in_place()` returns
`+1`. The native row reducer swaps two rows and returns only the rank; the
determinant caller consequently loses the swap parity. No column swap cancels
that sign. The same implementation remains in freshly checked upstream
`community` at `f4e787074d45f1dddd0b9646a5ebc85690deb2d1`.

A narrow fix is published as
[Symbolica PR #61](https://github.com/symbolica-dev/symbolica/pull/61), commit
`74d712d0e50692012c9c45778a06efd72f7c33ef`, authored and published as
`ValentinHirschi`. It preserves the public row-reduction API and shares the
existing elimination. The freshly compiled upstream baseline fails the two
odd-swap controls. With the patch, all five new regressions, 176 existing
Numerica unit tests and 14 existing API regressions pass: 195 distinct tests,
zero ignores. Tests use a coherent cached native GMP/MPFR dependency graph
through direct `rustc`; changed files pass rustfmt and whitespace checks.
Root independently reviewed the patch before publication. GitHub denied formal
reviewer assignment for the contributor account, so review was explicitly
[requested from `benruijl` in a PR comment](https://github.com/symbolica-dev/symbolica/pull/61#issuecomment-6094531274).

FastSecDec's current contour determinant calls `det()`, which
already tracks row-swap parity; this defect does not explain its larger-chart
memory cost and does not require changing the consumer dependency pin.

The generation agent independently reviewed the structured determinant proof,
native probe and stated evidence limits. Actual higher-dimensional physical
mapping and implicit-root jet acceptance remain separate from this audit.

## Integration gates before any production change

Keep the existing strength callback and all mathematical coordinate dependence.
Differentiate before imposing subtraction faces. Preserve coefficient-only
native definitions, actual-face request associations, exact-offset native
materialization and v12 state export. A determinant representation must not hide
strength callbacks behind a new opaque function or duplicate a solver/AD in
FastSecDec.

Any candidate should first match the current determinant and its derivatives on
small exact examples, including zero full determinants and endpoints; then
match actual eight- and nine-dimensional source charts with native high
precision derivatives and numerical determinants. Measure both retained Atom
size and native IR size, and bound per-chart staging/residency. No full generic
factorial template cache or unbounded chart metadata retention is justified by
this audit. An upstream improvement is warranted only for a concrete owner
defect or missing operation established by source and an executable reproduction.
