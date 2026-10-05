# Native fullgraph persistence memory review

Independent review on 2026-10-05 of the prepared original on-shell attempt and
the existing persistence path. This is a failure diagnosis and reuse review,
not completed full-integral or performance acceptance. No scientific process,
production edit, new codec, or dependency patch was introduced by this review.

## Accepted terminal evidence

`output/diagnostics/native-named-prepared-fullgraph-3` contains all 1,026
coefficient-completion events and the final 1,026/1,026 sector-compilation
callback at 1,668.039794677 seconds. Completed coefficient-expansion work totals
1,531.885429222 seconds. The CLI earlier reported eight active parameters and
one term; no finished artifact exists to supply its authoritative preparation
report.

The process then reported `memory allocation of 8388608 bytes failed`. Before
the 1,800-second deadline, an independent passive observation found it dumping
core with virtual size 31,456,076 KiB under the recorded 30-GiB address-space
limit. The watchdog subsequently interrupted it and killed it after the grace
period. The final timer honestly records `timed_out: true`, signal 9, wall time
1,807.811116401 seconds and peak resident size 26,771,064 KiB. Both the original
allocation failure and subsequent watchdog outcome matter; this is not evidence
that coefficient generation remained unfinished until its deadline.

All 60 immutable hashes were independently rechecked. The wrapper was reaped,
all three owned process IDs were absent, and no artifact, inspection,
integration, checkpoint, or saved result was produced. No manual signal was
sent. The review record is
`output/diagnostics/native-named-prepared-fullgraph-3/independent-terminal-review.json`,
SHA-256 `552de8975a2b527df25034ab71f9f0f53a1a5f727821de47b5d86276ea889307`.

The final sector callback precedes `KernelSet::finish`, which initializes the
portable kernel envelope before the CLI constructs and saves its outer
artifact. Consequently, the evidence localizes failure **after sector JIT
compilation and before publication**, but does not identify the precise native
envelope, CLI envelope, or file-save allocation. A claim that the final file
writer itself failed would exceed the retained evidence.

## Existing owners and avoidable copies

Native Symbolica owns exact evaluator serialization and structural validation.
Its `ExpressionEvaluator` implements ordinary Serde serialization using
borrowed fields; decoding retains the existing `validate_structure` check.
FastSecDec uses that existing Serde representation through Bincode 2.0.1.
Neither an instruction format nor a serializer implementation is missing.

| Existing path | Source-observed allocation |
| --- | --- |
| `kernel/artifact/native.rs::compiled` | Copies every retained program byte array into an owned payload. |
| `native.rs::content_id` | Serializes the complete payload into a temporary JSON vector only to hash it. |
| `native.rs::encoded` | Produces the complete retained JSON envelope vector. |
| `KernelSet::to_bytes` | Clones the retained complete envelope. |
| CLI `Artifact::new` | Parses the cloned envelope into `serde_json::Value`, allocating a general JSON value for every native program byte. |
| CLI `Artifact::identity` and `save` | Allocate another compact JSON vector for hashing and a complete pretty JSON vector for writing. |
| CLI `Artifact::load_with_preflight` | Reads the entire file, builds the full general JSON tree, and serializes the kernel tree back into another vector before native decoding. |
| `native.rs::load` | Rebuilds a native artifact during `from_programs_with_progress`/`finish`, then replaces that envelope with a copy of the original bytes. |

These are concrete opportunities to reduce memory without changing numerical
programs, aliases, graph inputs, coefficients, maps, conditioning, or replay.
This inspection does not quantify each allocation's contribution to the failed
run, nor promise that removing one copy alone closes the fullgraph gate.

## Reuse boundary for a narrow fix

The locked `serde_json` 1.0.151 already provides `to_writer`,
`to_writer_pretty`, and `from_reader`; `std::io` supplies buffered file I/O.
FastSecDec's existing `sector_content_id` already streams Serde JSON directly
into a `blake3::Hasher`. The same pattern can hash native payloads and CLI
provenance without allocating a complete temporary JSON vector. Borrowed
Serde views can reuse native program slices rather than clone them.
Atomic file replacement can retain its current create-new temporary file,
flush/sync, rename, and error cleanup while accepting streamed output.

Serde JSON also supplies `RawValue` behind its existing `raw_value` feature
when a retained nested JSON fragment is needed. It must not be substituted
blindly: the current CLI scientific identity hashes a `Value`, whose object
keys are sorted and whose whitespace is normalized. Native struct field order
and a raw fragment from an old pretty-printed file differ. Preserve the old
canonical identity semantics and strict legacy loading while removing the
large per-byte `Value` representation. A duplicate native evaluator schema in
the CLI would be an avoidable ownership problem; the library should own the
portable kernel transport boundary.

Bincode already provides `serde::encode_into_std_write` and
`serde::decode_from_std_read` if future caller streaming requires them. There
is no reason to switch the present native codec merely to remove outer JSON
copies. Retain the existing codec/version, dependency bindings, structural
admission, complete metadata and exact native program bytes.

Validation should reuse the existing artifact identity, legacy compatibility,
tamper rejection, cold-load and replay tests. Focused byte/identity comparisons
and a bounded transport-memory check can establish that the proposed change
actually removes the identified copies. The existing fullgraph capability
trial remains the end-to-end acceptance task; this audit adds no optional
performance campaign or mathematical gate.
