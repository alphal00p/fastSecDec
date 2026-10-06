# Validate decoded native evaluator IR

Status (2026-10-06): this optional decoder-hardening patch is removed from
delivery. Native evaluator artifacts are trusted application caches; FastSecDec
retains envelope and compatibility checks without claiming validation of an
arbitrary rewritten instruction stream. The historical malformed-IR probe
below was never a failure of FastSecDec generation.

## Historical proposal and validation

Status: independently source-reviewed; four focused native tests and the
preserved-byte before/after codec checks pass. FastSecDec's public pipeline gates
are tracked separately by the coordinator. The unchanged upstream base is
Symbolica 3.0.1, revision
`98794d0d7337ba2b08e4c046dde584ad7fc1ce10`.

The serde and bincode evaluator decoders previously accepted unchecked internal
stack partitions, operand/result indices and instruction shapes. Safe public
evaluation assumes those invariants, including unchecked Add/Mul accesses.
Outer checksums and matching public input/output lengths do not establish them.
No existing public native validation API was found in the evaluator modules.

The safe standalone probe `output/probes/native_ir_decode.rs` uses only native
Symbolica and codecs. It serializes six malformed layouts and **never evaluates,
exports or maps malformed IR**. The four-patch native library accepted every
case via serde, native Decode and native BorrowDecode: empty operands; operand,
destination and result outside the stack; reserved partition past the stack;
and parameter count past the reserved partition. Original JSON and native
bincode fixtures are frozen under
`output/diagnostics/native-ir-decode/malformed-bytes`, with a SHA-256 manifest.
The original executable and dependency hashes remain in the adjacent evidence.

The isolated patch adds native
`ExpressionEvaluator::validate_structure() -> Result<(), String>` and calls it
from each decoder before returning an evaluator. One native helper covers root
and function bodies. It validates partitions, results, operands, writable slots,
nonempty Add/Mul lists, complex phase prefixes, supported builtin operations,
forward label targets, function arities and earlier-callee ordering, external
constant slots, and populated worker-body stack sizes. The serialization layout
is unchanged. External tag parsing now uses the existing fallible native parser
so malformed text becomes a decode error rather than a panic.

This is representation validation, not expression equivalence, external-code
validation or reconstruction of the originating graph. It introduces no
FastSecDec instruction schema, optimizer or interpreter. Valid native branch,
zero-argument and nested-function programs remain supported. Native tests never
execute invalid data; their valid controls evaluate both branches and roundtrip
an evaluator after its native worker stacks have been populated.

The patch file is isolated against the already reviewed four-patch worktree:
it excludes the earlier fixed-argument external-constant changes in the same
`external.rs` file. `git apply --reverse --check` succeeds against the changed
worktree. Frozen earlier reference, baseline and sample-latency executables are
not rebuilt or relabelled by this source change. FastSecDec version-three
artifact loading is gated on the native validation and public cold-load tests.

`cargo test -p symbolica --lib evaluate::validation::tests --locked --
--test-threads=1` passed all four tests, with no ignored tests, after the native
unit-test build. The log is `output/symbolica-ir-validation-native-tests.log`.
The same isolated patch passes a forward application check against saved
pre-patch files as well as the reverse check against changed files. The native
test-only zero-argument builder needed an explicit `Vec<Symbol>` type; this was
a compile-time test setup correction, not a production change.

The rebuilt normal dependency is
`target/debug/deps/libsymbolica-1f655858224cb588.rlib`, SHA-256
`01601b6df1747703693fdbf78818f9bf2bb74f4e4b1a888a5971b678cc60b61a`.
Against that library, the standalone `--cfg patched` probe rejects all six
original JSON and native-bincode fixtures through serde, Decode and BorrowDecode
(18 successful rejection checks). The original byte manifest still passes;
`output/diagnostics/native-ir-decode/result-fixed.txt` records the outcome. The
previous four-patch optimized-debug library was
`dda215a39b38358772677beed211e773c9b028f33f0251dc1072135fe37f2e51`.

A separate 33-byte malformed exact-rational program was produced from the
native public `x` evaluator using the frozen four-patch release library: its
single result index was changed to 99 through native serde before encoding.
The old decoder accepted it; it was never evaluated. Its SHA-256 is
`20b3746faff76e5a045f6d4db176509a4ad52fb6a4d9778dff38e73884175a62`.
The original serde form, generator source, frozen library identity and complete
record are under `output/diagnostics/native-ir-decode/exact-fixture*`. It provides
a public FastSecDec loader regression without a test-local instruction encoder.

The public regression subsequently passed in the five-test artifact follow-up
(`output/native-ir-artifact-followup-tests.log`). It inserts those unchanged
native bytes into a genuine one-input/one-output v3 artifact, recomputes the
outer identity, and receives a native evaluator decode error before backend
compilation. This confirms that rejection is structural, not merely a checksum,
truncation or public-dimension check. The coordinated initial production science
gate also passes 103 tests; final combined gates remain separately recorded.
