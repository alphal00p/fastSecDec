# Native direct-output facts: source-only options

The v3 loader currently uses conservative zero and component-realness facts.
That cannot suppress a needed precision check, but a padded zero or exactly real
output in a complex vector can cause extra rescue when an amplifying weight
triggers range checks. This note identifies an existing native API option for a
bounded follow-up. It introduces no implementation or dependency change and
makes no measured performance claim.

Reviewed owner: Symbolica 3.0.1 at
`98794d0d7337ba2b08e4c046dde584ad7fc1ce10`, including the reviewed structural
decoder patch. Relevant native files are `src/evaluate/instruction.rs`,
`evaluator.rs`, `optimize.rs` and `tree.rs`.

## What the public native interface provides

`ExpressionEvaluator::export_instructions()` returns native
`ExportedInstructions`, including typed `Slot::Const`, `Slot::Out`,
`Instruction::Assign`, exact constants and `constant_functions`. The native
exporter appends an assignment for each result whose original stack index is a
parameter or constant, and for repeated result slots. Raw evaluator result
indices remain private. `get_constants()` alone therefore cannot associate
constants with particular outputs.

For a structurally validated evaluator, an exported direct
`Assign(Out(output), Const(index))` whose index is absent from
`constant_functions` identifies an exact literal output. The native validator
forbids instruction destinations in the reserved parameter/constant partition,
so runtime operations cannot overwrite that literal slot. Existing exact
rational predicates can establish that this literal is zero, or that its
imaginary part is zero. All other outputs may remain unknown. This requires no
operation evaluation, recursive expression restoration, constant propagation,
general instruction analysis or sampled zero test.

The external-slot exclusion is essential. Native
`ExportedConstantFunction` explicitly says these slots can contain rational
placeholders awaiting fixed-argument or registered-function evaluation at the
requested precision. Their current zero value must not be treated as a proof.
Similarly, a non-inlined function, an arithmetic result, a branch or a duplicate
`Out` assignment should remain unknown under this deliberately narrow option.

## Existing alternatives do not supply the same certificate

`ExpressionEvaluator<Complex<T>>::is_real()` checks stored stack coefficients;
it does not certify the reality of every output under every domain or external
function. `set_real_params()` propagates native phase information using explicit
assumptions about logarithms, roots, powers and custom functions, but returns no
per-output semantic certificate. Serialized phase hints must not become new
trusted zero/real claims merely because their structural shape is valid.

The public vector `InstructionList::is_zero()` already excludes unknown
constants, but applies to that distinct native construction type and does not
provide a borrowed view of an arbitrary decoded scalar evaluator. Rebuilding an
instruction list solely to obtain facts would add an unnecessary representation.
The native randomized expression `zero_test()` is not an exact output-zero
certificate and is not needed here.

## Cost and acceptance boundary

Export currently allocates/clones the full root instruction stream, constants
and referenced function bodies; shared bodies are retained through native
`Arc`s within the exported representation. This is a one-time load/build cost,
not an allocation for each sample. The absence of a borrowed result-index
accessor does not presently justify another native patch. Any follow-up should
first measure actual cold-load and weighted-rescue cost on the same validated
programs, then compare that saving with the export allocation.

A proposed implementation would need separate controls for literal zero and
real/complex constants, registered/fixed-argument placeholder constants,
arithmetic cancellation that remains unknown, repeated outputs, and malformed
data rejected by the native decoder before exporting. Complete vectors and
weighted fresh/cold results must agree without relaxing precision policy.
Until those measurements and controls justify a change, conservative cold
facts remain the accepted production behavior.
