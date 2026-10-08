# Optional kernel-load validation (2026-10-08)

`KernelLoadOptions { validate: false }` is the default for loading saved native
programs. The options-aware APIs also accept `validate: true`, with or without
the existing caller-owned progress callback. The binary and native JSON wire
formats are unchanged.

Optional validation recomputes the binary transport digest, semantic/native
JSON identity, polynomial-support admission and exact chart geometry/maps.
Ordinary loading restores the stored Atoms and coordinate images/Jacobian
directly. It retains header/version/codec, identity syntax, byte exhaustion,
numerical policy, input/output layout, coordinate dimensions, chart/kernel
associations, cancellation-profile and runtime-mass checks. It does not rerun
threshold certification. Both paths decode Atoms through Symbolica's existing
state context and restore saved evaluator programs without expression rebuilding.

The outer binary envelope borrows its symbol-state and payload buffers from
the input bytes, avoiding an additional complete payload allocation. Decoded
sector program bytes move into retained `Arc<[u8]>` owners and accompany their
decoded native IR into backend restoration; this avoids reserializing the same
programs. Backend preparation still follows the existing eager/SymJIT owner.
This change does not claim to persist executable machine code.

## Focused evidence

Native checks passed with the Apple Clang linker:

- `cargo test --locked -p fastsecdec --lib kernel::artifact::`: 21 passed.
  Includes options/default behavior, binary and native JSON hash switches,
  mandatory layout/exhaustion checks, borrowed-envelope wire/pointer checks,
  progress/cancellation and numerical/metadata parity.
- `cargo test --locked -p fastsecdec --lib kernel::metadata::`: two passed.
  Shape and association errors remain rejected in both modes. Invalid retained
  proofs require explicit validation to reject. Valid metadata agrees between
  both modes, including projective and signed positive-orthant coordinate maps.
- `cargo test --locked -p fastsecdec --test kernel_artifacts`: ten passed.
  Scientific cold replay, backend settings, inspection metadata, immutable
  native bytes and literal-zero rescue remain covered. The existing corruption
  assertion now explicitly requests validation; both modes retain format and
  byte-exhaustion assertions.

Logs are ignored under `output/optional-validation-review/core-*.log`.
These controls do not establish load-time performance for the user's large
three-loop artifact. Independent native-cache/loader review is recorded in
[the SymJIT loading review](symjit-cache-loading.md).

## Milestone checks

The complete native core library suite passed 146 tests, with 16 pre-existing
ignored tests. The standalone portable/eager suite passed all 72 tests,
including cold-process artifact replay, complex covariance, numerical-dual
Taylor/IBP evaluation, runtime MC/QMC and weighted precision replay.
Both generation integration-test targets also passed all 20 tests (16
generation and four retained-metadata tests). Their existing corruption checks
now explicitly request validation; ordinary valid round trips keep the default.

CLI checks passed 69 unit tests (one pre-existing ignored test) and both
inspection process tests. They cover default/explicit validation, mandatory
JSON/binary identity agreement, metadata-only inspection without a data file,
resume preflight and complete MC mean/error/covariance equality between loading
modes. Strict workspace/all-target Clippy passed. Logs are ignored under
`output/optional-validation-review/`.

The release CLI build passed in 1 minute 35 seconds with Rust 1.99 and the Apple
Clang linker. Formatting and whitespace checks passed. The artifact format is
unchanged, so these loading improvements require rebuilding the CLI, without
regenerating compatible saved artifacts.
