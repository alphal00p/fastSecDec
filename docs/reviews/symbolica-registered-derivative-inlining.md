# Registered derivative bodies with native Never inlining

The native inlining investigation exposed a Symbolica lowering error: a tagged
`DERIVATIVE` function registered with an actual native body worked with
`InliningPolicy::Always` but failed with `Never`, reporting
`UnsupportedBuiltinArity { function: der, expected: 1, actual: 4 }`.
The registered body had already been compiled; final instruction lowering
selected the fixed builtin path before consulting that body.

[Symbolica PR 65](https://github.com/symbolica-dev/symbolica/pull/65) fixes this
in the owner, from upstream community base
`f4e787074d45f1dddd0b9646a5ebc85690deb2d1`, commit
`31fd50e8f2d9b36303741162cd82dcae647b44ae`. It was authored and published as
`ValentinHirschi`. Lowering uses an exact registered symbol/tag entry with an
actual body before builtin-only handling. Unregistered builtins retain their
existing precedence and arity checks. No public API, algebra or codec changes
were introduced.

The native API/source investigation checked FunctionMap registration,
`InliningPolicy`, body compilation and backend conversion. Existing backend
converters already distinguish body-bearing builtin symbols, supporting this
narrow correction. Independent source/test review accepted the routing and its
ordinary-builtin boundary before publication.

Validation used a coherent native owner dependency graph and direct Rust
builds. The baseline tree retained the upstream lowering implementation; its
library also included the unrelated complex-serde bound correction from PR 64.
The new builtin control passed and both derivative controls failed with the
reported arity error. After the fix, all three controls passed. They compare
explicit complex polynomial values and first, mixed and third derivatives for
Always/Never, verify the presence of a native subevaluator only for Never, and
restore owned exact and SymJIT O2 programs after dropping the original owners.
Ordinary LOG precedence/arity and unregistered derivative refusal are covered.
The existing evaluator suite passed 15 tests; its existing 1,000-variable test
remained ignored for its debug stack-overflow guard. Formatting and diff checks
passed. CUDA-named existing controls are not evidence of GPU execution.

The formal GitHub reviewer assignment was denied by repository permissions.
An explicit `@benruijl` review request is recorded on the PR; no formal reviewer
assignment is claimed. The PR is attached to the task.

This owner fix is **not adopted into FastSecDec's dependency pin**. FastSecDec's
current Always-inlining policy and all running generation artifacts are
unchanged. The small native probe establishes capability and a routing defect,
not an improvement for physical generated expressions or sampling performance.
Both contour-Dual composition paths still require their existing inlining
contract. Any future Symbolic-J Never-inlining choice needs separate physical
generation, saved-owner and runtime evidence.

Ignored reproduction evidence is in
`target/symbolica-registered-derivative-never/` (`library-command.json`,
`baseline-r2.log`, `patched.log`, exact test commands and `fmt.log`). The original
bounded capability failure is retained in
`target/contour-definition-traversal-probe/inline-result.log`. No raw output,
reference worktree or compiled artifact is tracked.
