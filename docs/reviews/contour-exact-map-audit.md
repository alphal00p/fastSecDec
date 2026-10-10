# Dynamic exact-vector evaluation: native cache audit

Independent review, 2026-10-10. The exact vector must first be assembled as
native mathematical Atoms, preserving cancellations between independently
generated records. Compiling each record first and subsequently summing its
numeric values or composed evaluator outputs does not preserve removable
singularities. The native probe with opposite `1/p` terms gives finite summed
Atoms at `p=0` and nonfinite separate/composed program results. No replacement
algebra or local evaluator is justified.

## Existing public operation and its limit

Symbolica `AtomCore::evaluate_with_prec` accepts full custom-function Atoms as
map keys. Its implementation checks that map before resolving a custom
callback, and clones the supplied native numeric value. The owner tests cover
function-value map input; executable probes additionally confirmed reuse in
f64, DoubleFloat, Float, tracked f64/Float, complex f64 and complex tracked
domains, retaining nonzero tracking errors. Canonical mathematical Atom
equality is sufficient; diagnostic namespace tags must not enter the summed
mathematical expressions.

However, eagerly filling that map from all syntactic root occurrences is not
a general solution. Native `if` evaluates its condition and only the selected
branch. An unreachable invalid root must not execute or be certified as an
executed candidate. A prepass over both branches changes that behavior.
Nested root functions likewise require the owner's actual evaluation order;
an arbitrary prepass can solve an inner function more than once. Restricting
input syntax merely to simplify a diagnostic prepass is unnecessary when the
native evaluator already implements these semantics.

`evaluate/tree.rs` already has the needed behavior: its private recursive
`evaluate_impl` accepts a native function-value cache, evaluates branches
lazily and stores executed registered custom functions. `AtomCore`'s public
direct entry point creates a new cache for every expression. The public API
and source search found no direct vector evaluation API exposing that cache.
The existing compiled evaluator/composer path is a different operation and
does not fix the cancellation issue described above.

The focused executable evidence is retained in ignored workspace probes
`target/contour-exact-function-map-probe.*` and
`target/contour-exact-cancellation-probe.*`. These demonstrate both the useful
public function override and its lazy-branch limitation; they do not justify
a general eager-prebinding claim.

## Accepted narrow owner API

Expose a single direct native vector evaluation call, with one immutable input
map, numeric domain and precision:

```rust
evaluate_multiple_with_prec(expressions, point, bits)
    -> Result<DirectEvaluation<'expressions, T>, EvaluationError>
```

The result owns the numeric vector and the existing native cache, whose keys
borrow the unchanged expression atoms. Expose `values()`, `into_values()` and
a lazy `function_values()` iterator over actually cached function values.
Reuse the existing recursive implementation; do not add an evaluator, branch
walker or callback cache in FastSecDec. A single call cannot reuse a cache at
a different physical point/precision; new precision attempts create a fresh
native call. Returning a borrowed-key cache avoids atom/value copying and
inspection work when runtime validation is off.

The function-value view must explicitly describe what native caching records:
registered user-function executions, including tagged/constant functions;
not every built-in operation, not caller-supplied function-map overrides and
not unselected branches. Native errors/nonfinite behavior remain unchanged.
FastSecDec continues to fence callback failures around the entire attempt.

Required owner regressions include shared calls across outputs, nested calls,
unselected invalid branches, partial and total native cancellation before
evaluation, callback tags, full-map precedence, tracked/complex native values,
and independent repeated calls at different points/precisions. FastSecDec
then matches executed mathematical root keys to merged namespace/face
associations and certifies the actual candidate values without a second solve.


## Owner delivery and independent review

The operation is implemented in [Symbolica PR 59](https://github.com/symbolica-dev/symbolica/pull/59),
head `7553de426d63628f679ed6e7b4a95acb834c8d76`, against the upstream
`community` base `f4e787074d45f1dddd0b9646a5ebc85690deb2d1`. The four-file
change introduces the result/API module, reexports it, widens the existing
recursive method only to its parent module, and adds focused integration tests.
No optimizer, serializer, numerical type or new expression evaluator is added.
Both the coordinator and the independent foundation reviewer inspected the
actual native recursion/cache semantics and accepted this scope.

The pre-change import fails with E0432; the direct owner build succeeds with
the cached native dependency graph. All **8 new tests pass**, including lazy
invalid branches, nested callback reuse (five separate scalar executions become
two shared vector executions), exact prior cancellations, map precedence,
tagged/constant identities, point/precision isolation, nonfinite behavior and
f64/DoubleFloat/Float/tracked/complex domains. The existing owner evaluation
integration suite also passes **15 tests, with one existing stress test
ignored**. These are focused direct native build/test results; they are not a
claim that the complete owner Cargo suite ran. The owner build emitted only
an existing unrelated unused-result warning in `src/atom.rs`.

The commit author and authenticated publisher are `ValentinHirschi`
(`valentin.hirschi@gmail.com`). GitHub denied a formal reviewer assignment due
to `RequestReviewsByLogin` permissions; the authorized
[@benruijl review invitation](https://github.com/symbolica-dev/symbolica/pull/59#issuecomment-6091614060)
is a comment, not a formal assigned review. The PR is attached to the task.

Consumer adoption keeps the existing public `7ec1be4` dependency lineage:
only this narrow API commit is cherry-picked, preserving the existing native
composer and all previous contour fixes. It does not adopt the latest upstream
branch wholesale. The public combined revision is
[`1e1cb169bec35ed3b8536050f789321f063a2047`](https://github.com/ValentinHirschi/symbolica/commit/1e1cb169bec35ed3b8536050f789321f063a2047),
branch `codex/contour-direct-vector-consumer`. The same focused direct build
and tests pass there: **8 new tests plus 15 existing evaluation tests, one
existing stress test ignored**. The only cherry-pick conflict was the module
list; the existing composer and new direct module are both retained. Its diff
against `7ec1be4` remains exactly the four API/test files. FastSecDec dependency
updates remain coordinator-owned; the checked exact-offset consumer is the
next separate implementation step.
