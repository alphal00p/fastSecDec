# Native evaluator composition: independent review

## Scope and ownership

This reviews the proposed Symbolica extension in the isolated
`output/numerical-dual-study/symbolica` checkout: `evaluate/compose.rs`, its
public reexport, and the extraction of existing scalar instruction lowering
from `evaluate/dual.rs`. The reviewer did not author or modify those files.
The review initially used a local prerequisite. The exact reviewed revision was
subsequently published through
[Symbolica PR #54](https://github.com/symbolica-dev/symbolica/pull/54), targeting
`community`, and pinned from the public fork in all three FastSecDec workspaces.

The missing operation is connecting the outputs of one existing evaluator to
the inputs of another without reconstructing a symbolic expression. The public
composer reuses native `ExpressionEvaluator`, `InstructionList`, `Slot`, the
existing scalar inliner, constants/callback ownership and existing optimizers.
It introduces neither a second interpreter nor an algebra/series implementation.
The older independent-output merge API does not provide arbitrary input binding.

## Source findings

- `append` checks exact input arity, slot ranges, unsupported output slots and
  non-inlined function bodies before changing composer state. Every explicit
  `Result::Err` path precedes mutation. Accepted source evaluators are cloned;
  their original stack, instructions and outputs remain unchanged.
- Native stack unoptimization gives the inliner distinct instruction results.
  It maps source parameters to caller-provided slots and appends constants and
  temporary storage after existing work. Repeated inputs, fanout, reordered
  outputs, parameter outputs and constant outputs retain their meaning.
- Precision-dependent external constants retain their native callback and
  relocated constant index. Callback registry entries are remapped with their
  native symbol/tags/arguments; constant locations participate in deduplication.
  Non-inlined sub-evaluator bodies are explicitly rejected rather than dropped.
- Every appended program receives fresh branch labels. Native final label fixing
  resolves the assembled stream after optimization. Conditional programs retain
  their native branch and join semantics; no branch is replaced by eager
  evaluation of both alternatives.
- The extracted scalar lowering preserves parameter/constant/temp offsets,
  instruction argument ordering, selected outputs and callback ownership.
  Existing vectorization still requests the same single common-pair pass;
  composition uses its caller's configured common-pair limit. Native stack
  optimization and label repair remain the owners of those transformations.
- The implementation has no backend gate, worker pool or hidden execution.
  Eager consumers can use the same exact scalar program; SymJIT lowering remains
  optional and is tested separately below.

`Slot` is an existing raw native index. The documented contract requires inputs
or returned outputs from the same composer; range validation is not a claim to
prove ownership of a forged, same-valued slot from another composer. Likewise,
this review assumes valid native evaluator objects, not manually corrupted
private instruction storage. No finding requires a new opaque slot API or a
new validator for trusted native serialization.

No correctness finding was identified in the initial reviewed source. The
follow-up optimization is reviewed below; neither review claims optimal
instruction count for arbitrarily large composed programs.

## Independent executable controls

A separate ignored probe, `output/numerical-dual-review/composition.rs`, links
against the freshly built native owner. It compares the composed evaluator with
separate calls to the original native programs, not an alternate algebra engine.
It passes these controls with both eager evaluation and SymJIT O2:

- Two separately appended nested-conditional programs, with both conditions
  repeatedly toggled across calls, exercise independent labels and reused stacks.
- Shared/reordered inputs, fanout, repeated outputs and direct parameter outputs.
- Runtime `gamma(x+1)` callbacks and precision-dependent `gamma(1/2)` constants.
- A negative rational numerator larger than 128 bits and zero-input evaluators.
- Invalid arity, each invalid slot class and a non-inlined registered function
  rejected after successful work; subsequent valid work matches the untouched
  source programs, checking failure atomicity.
- Empty output layout and rejection of an invalid final output slot.

Native output is retained in `output/numerical-dual-review/composition.log`.
The same eager controls also pass in a separate portable-feature build with
Malachite/Astro and no JIT. An additional real/complex composition control checks
both output components through inverse/power operations, the Gamma constant and
successively dependent native calls. It passes with native eager, native SymJIT
and portable eager. Portable evidence is retained in
`output/numerical-dual-review/portable-composition.log`.

The reviewed isolated owner commit is `9b82a0b` (native scalar evaluator
composition with output selection). These host-side numerical controls establish
native/portable arithmetic agreement on their fixtures; they do not establish
browser execution, whole-graph performance or complete FastSecDec lane acceptance.


## Follow-up: selected-output pruning and native CSE

The follow-up isolated commit is `1deccb8538ccb91dc2c1e58fc0a2e900d2276bf4`.
Its composer-only pass operates on native scalar instruction storage,
not symbolic expressions. `append` first calls the existing stack-unoptimization
owner, so each temporary read refers to an earlier distinct write. A reverse
slice starts at the selected outputs, marks every input of a live instruction,
and then removes dead instructions, constants and callbacks. Rebuilding in
forward order remaps earlier dependencies before their consumers. Direct
parameter/constant outputs and repeated outputs are retained. Any branch, label,
jump or join bypasses this straight-line pruning; it is deliberately not a new
control-flow optimizer.

Precision-dependent callback constants are distinguished from literals by the
same symbol/tags/fixed-arguments key as native evaluator merging. Their cached
placeholder value cannot merge with literal zero or with a different callback.
The pass relocates constant indices, external-function indices and output slots,
retaining callback ownership. Ordinary external calls remain distinct from
constant callbacks. The new exact-value equality/hash bound matches the native
merge owner's requirement and is satisfied by FastSecDec's exact scalar IR.

After pruning and constant deduplication, composition delegates common
instruction elimination, common-pair elimination, stack optimization and label
repair to existing Symbolica implementations. It does not add an alternative
CSE engine. Shared lowering receives an explicit composer-only switch; the
existing Dualizer path keeps its original single common-pair pass, including
its original abort-level behavior. The independent review checked that final
baseline-preservation correction in source.

An expanded independent native probe passes the original eager/SymJIT controls
and selection subsets spanning integer/floating powers, builtin/runtime
callbacks and arithmetic, repeated literals, distinct Gamma constants, dead
callback owners, reordered/repeated/direct outputs, and CPE limits 0, 1, 5 and
unlimited. Conditional `log(0)` alternatives remain unevaluated when not
selected, even after repeated branch and stack reuse. Evidence is in
`output/numerical-dual-review/composition-new.log`. The same expanded controls
pass with the final commit's portable eager build and registry Numerica/Graphica:
`output/numerical-dual-review/composition-portable-new.log`. This final consumer
check supersedes the earlier isolated portable fixture's local numeric patches.

The portable validation manifest now includes the existing native scientific
`numerical_dual` target verbatim and its direct test-only sectors dependency.
All 11 controls pass with Malachite/Astro and no JIT (1.53 seconds of test time).
They cover both subtraction strategies, mixed faces, runtime points, binary
replay, exact inspection, retained/reordered caller execution, cancellation,
explicit local symbolic fallback and zero-dimensional exact contributions.
Isolated Python binding and `python_stubgen` feature checks also pass against the
same local prerequisite and current core. Locked all-target binding Clippy with
`python_stubgen` and warnings denied also passes. Logs are retained under
`output/numerical-dual-review/`. No Python wheel or browser run is claimed for
these new options.

Temporary local Symbolica patches in the root, portable and binding manifests
were used for this implementation evidence. Original portable/binding manifests
and locks were backed up under `output/numerical-dual-study/workspace-original`;
only Symbolica was overridden in the consumer dependency graph. Publication
preserves that exact source revision. The manifests now select its public Git
source; locked checks of that source identity are recorded in the delivery audit.
