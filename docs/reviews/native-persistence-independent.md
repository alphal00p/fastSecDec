# Independent native persistence review

The source change addresses the avoidable copies identified in the
[memory audit](native-persistence-memory-audit.md). Source and focused test
review found no unresolved issue. The 16-test native and 42-test CLI focused
runtime gates pass. The combined workspace gate also passes: 380 tests passed,
none failed, and 23 remained ignored; formatting and all-target Clippy pass.

## Native ownership and compatibility

The library retains the same version-three schema, native Symbolica evaluator
Serde/Bincode codec, compiler identity, metadata and precision policies.
Borrowed payload and sector views use the original field order and reuse program
byte slices and cancellation terms. `serde_json::to_writer` streams precisely
the previous JSON representation into the existing BLAKE3 digest. This removes
the full temporary hash buffer and program-byte copies without adding a parser,
serializer, instruction format, CAS operation or dependency patch.

`KernelSet::artifact_bytes` borrows the retained complete artifact. The existing
`to_bytes` API still returns an owned copy. Fresh compilation initializes the
artifact once. The renamed private loader constructors finish the same numeric
and semantic admission without encoding an envelope that would immediately be
discarded. Both native and legacy loaders then retain the original validated
bytes and content ID. Independent call-site review confirms those constructors
are loader-only. Native structural validation, exact-byte exhaustion, parameter
and output layouts, real/complex admission, metadata validation, exact offset
evaluation and finite-value rejection remain in their original owners.

The new transport test compares borrowed serialization and streamed hashing
against the previous owned payload/envelope and whole-buffer hash. It exercises
every byte value, escaping, signed orders, components and optional cancellation
metadata. Existing scientific artifact tests additionally compare generated and
compiled version-three bytes, load the committed version-one/two fixtures,
reject malformed native codecs and IR, and replay complete complex Gamma
vectors in separate processes. The additional pretty/reordered-byte test
checks semantic identity and exact retention of originally loaded bytes.

## CLI envelope and caller boundary

The CLI keeps the native envelope opaque using existing Serde JSON `RawValue`.
It does not introduce a second native evaluator schema. Newly produced outer
artifacts use an explicit version-two identity: the existing provenance
serialization plus the native kernel content ID, under a distinct domain tag.
The native loader must still verify the actual payload against that claimed ID
and perform structural admission before returning successfully. This is not a
way to trust an unchecked embedded hash.

Version-one outer artifacts retain their previous canonical `Value` hashing
branch, original identity, and cold loading. That compatibility branch can
still allocate the historical general JSON tree; the current version-two path
avoids it. Existing outer identities are not silently reinterpreted. Scientific
kernel identities and bytes remain unchanged; new outer identities intentionally
change with their explicit version. Observation fields remain excluded.

Dependency identity checks and caller preflight precede kernel compilation.
Atomic output now streams through an ordinary buffered file writer, retaining
create-new temporary files, flush/sync, atomic rename and failure cleanup. The
generated symbolic object is released after successful compilation, before CLI
envelope construction. Additional status details distinguish envelope
preparation from file publication without coupling display to the library.

Four focused CLI controls independently reconstruct the previous version-one
hash, verify load/resave and complete vectors, check current native bytes and
observation exclusions, reject payload tampering even with both claimed IDs
unchanged, preserve caller-preflight ordering/errors, and exercise partial-write
cleanup. Same-process reload assertions are supplemented by existing genuine
cold CLI and native artifact process tests.

## Acceptance boundary

The native focused build completed successfully in 5 minutes 53 seconds. Its
four selected executables pass 16 tests with no failures or ignored tests:
five artifact/identity units, seven format/scientific artifact controls, two
cold-process controls, and two alias/precision-replay controls. Independent
review rechecked all 189 source bindings against both current files and the
retained source archive, all four executable hashes, raw test summaries and
successful process outcomes. The accepted record is
`output/diagnostics/native-persistence-1/independent-review.json`, SHA-256
`d93c2c433e3c6afac5f71e249e161bf037173938bccdb2ee64e2aee9e978b55d`.

The CLI focused gate passes 42 tests with no failures or ignored tests: 31 units,
four CLI process controls, two family-preparation controls, two coefficient
status controls and three dependency-provenance controls. Independent review
rechecked all 68 current/archive source bindings, six executable hashes, raw
summaries and all five successful process outcomes. The first build invocation
named a nonexistent test target and stopped before compilation; its log is
retained, and the corrected `dependency_provenance` target built successfully.
The accepted record is `output/diagnostics/cli-persistence-1/independent-review.json`,
SHA-256 `b4071ed82bb963a441d31be1975152fa676c9da9bc627e99ab5ab61dc4512a8d`.

The combined workspace result independently matches all 63 raw test summaries
and the recorded test, formatting and Clippy log hashes. All three stage records
report successful reaped processes. The union of the accepted focused source
maps contains 233 distinct files, all still matching their recorded hashes;
overlapping entries agree. Clippy retains the existing unused-`Result` warning
in the Symbolica dependency. The accepted record is
`output/diagnostics/persistence-workspace-1/independent-review.json`, SHA-256
`97156a5aed445e9326a1f8e74af7dab956c3e87dcbdd516a4176f25df8973b52`.

The persistence milestone's required validation is complete. The existing
bounded whole-graph capability trial will establish
whether the persistence capacity gap is closed; this source review does not
predict memory usage or claim completed full-integral agreement or timing
parity. No optional benchmark campaign or new mathematical gate is added.
