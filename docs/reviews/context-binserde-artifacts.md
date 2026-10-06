# Portable split artifacts and native context binserde (2026-10-06)

The CLI accepts one artifact basename, for example
`output/gghh_double_box.fsd`. It writes human metadata to the `.json` sibling
and expressions/evaluator programs to the `.dat` sibling. Neither file stores
a path to the other; loading derives both filenames in the same directory.
The two-file commit writes data first and metadata last. An interruption while
replacing an existing pair can leave a mismatch; loading rejects that mismatch.
The pair is not a transactional filesystem object.

## Ecosystem reuse evidence

Public Symbolica `58652fa` provides the required implementation directly:

- `atom/representation.rs`: `Atom: bincode::Encode` writes native storage;
  `Atom: Decode<C>` requires `C: HasStateMap` and renames through that map.
- `state.rs`: `State::export_partial` captures symbol definitions and dependent
  state; `State::import` returns the remapping context.
- `evaluate/evaluator.rs`: `ExpressionEvaluator` implements native binserde
  `Encode` and context-parametric `Decode`. External callback definitions and
  tags remain entirely owned by this codec.
- Its `packed_lengths_roundtrip` and `import_normalizes_after_remapping` tests
  demonstrate native Atom bytes and context remapping.

GammaLoop `259df87`, `processes/amplitude.rs` and `processes/process.rs`, uses
`bincode::decode_from_slice_with_context` with a context implementing Symbolica
`HasStateMap`; `GammaLoopContextContainer` additionally carries GammaLoop's
model. FastSecDec needs no model during kernel restoration, so its decoder uses
`StateMap` directly. Its metadata Atom wrapper supplies only canonical-string
JSON presentation: binary derivation delegates directly to Symbolica's native
Atom encode/decode. Every explicit Atom passes through native context-aware encoding during `.dat` serialization.
Precision/status enums use their existing serde adapters; they contain no Atoms.

A focused temporary Rust probe round-tripped an Atom and small exact evaluator
with exported/imported state: 91 bytes, `x^2+2` evaluated at `x=3` gives 11.
This small probe did not exercise large negative coefficients; the full ggHH
cold-load check exposed the numerical codec defect described below.
A full native kernel probe produced a 760-byte binary artifact. A separate
process registered 100 unrelated symbols before loading it, retained the same
semantic identity and evaluated `1/(1+x)` at `x=0.5` as `2/3`.
The temporary probe source is retained only in ignored output.

Native byte integrity and scientific identity are separate: process-local symbol
IDs and unrelated Symbolica state do not enter semantic identity. The semantic
digest reuses canonical Atom presentation and the evaluator's portable symbol
serialization. It also binds the ordered runtime parameter schema.

## Ownership and limitations

The numerical core owns binary serialization and semantic validation. The CLI
owns metadata filenames, provenance, relative source/reference locations and
filesystem writes. Native generation, evaluator preparation and sampling remain
separate caller actions. The backend compiler policy continues to distinguish
native SymJIT O2 from the portable Symbolica interpreter. Native IR caches remain
trusted application artifacts; byte hashes and layout checks do not validate
arbitrary rewritten instruction streams. Existing threshold labels can be read
as historical metadata without repeating threshold certification.

The source and reference paths emitted into artifacts are relative to the
artifact directory. Explicit run-card resume can relocate the card and input
directory together; content fingerprints still reject scientific source edits.
Result collision protection includes both siblings. Existing tests and other
examples are intentionally not migrated in this requested round.

## Large signed coefficients: numerical owner defect and safe admission

The first full ggHH cold load failed semantic identity validation. Investigation
and independent probes traced this to Numerica 3.0.1's GMP native binserde for
`Integer::Large`: `backend/integer.rs` writes `rug::Integer::to_digits` (absolute
magnitude) and reads `from_digits`, dropping the sign. A large negative rational
and large negative complex part changed value; positive large coefficients and
gamma/polygamma callbacks passed. The integrity check was not relaxed.

Evaluator fields therefore reuse FastSecDec's established Symbolica evaluator
**serde/bincode binary** codec, stored as an opaque byte vector inside the native
context-binserde envelope. Its borrowed decoder is required by Symbolica's
existing Symbol serde implementation. This preserves signed coefficients and
callback fixed arguments without a new numeric codec or dependency patch.
All explicit Atom fields continue to use native Encode/Decode<StateMap>, and a
large-negative Atom round trip passes. Interpreter and native compiler policies
remain distinct; cross-backend cache interoperability is not claimed. The format
header advances to version 5 so the rejected intermediate wire format cannot
be accidentally decoded with the replacement evaluator field schema.

Manual CLI checks passed for sibling-only loading after copying/renaming a
pair, missing-data rejection, data-integrity rejection, serial/two-worker
semantic identity equality, relative output with absolute CLI arguments, and
protection against writing numerical results onto the `.dat` sibling. Full
ggHH regeneration and cold-load evidence must use the corrected format.

The corrected full ggHH release run generated 30 kernels with 15 runtime
parameters in 48.34 seconds using eight workers. The metadata sibling is
14,314 bytes and the binary sibling is 2,113,944 bytes. A fresh process loaded
and inspected all kernels in approximately 0.565 seconds with the same
semantic identity; the complete folded exact offset remains explicitly deferred
until parameters are bound. Both siblings were checked for absence of absolute
host paths. This is generation/serialization evidence, not a convergence or
performance-parity claim.

The coordinator then completed a cold ggHH integration from this corrected pair:
122,880 points in 20.523 seconds, zero evaluation failures, 16,320 precision
rescues, maximum 256 bits and complete covariance. It stopped at the requested
work limit; no target-convergence claim follows from that bounded check.
