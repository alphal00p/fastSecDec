# Shared native dual helper extraction

The disconnected shared-helper repeat passed on 2026-10-05. No production
algorithm or actual captured coefficient has been accepted by this gate.

`output/probes/native_pole_dual_small.rs` remains byte-identical at SHA-256
`386ce084e431154e0281eac4d1e7ecf51c0049dd94c5478001bf30b42f2db2c4`.
Its eight native admission and precision helper bodies were extracted into
`output/probes/original_dual_adapter/native_core.rs`, SHA-256
`4be1d53b69ae54c28034bcc4a84680c9056ab63f3092a082985a77e4b6969c5f`.
Only visibility, imports and module ownership changed. The new small harness
retains the same source cases, native factor collection, native Dualizer and
unchanged native Series comparison. The independent reviewer checked all eight
bodies mechanically before execution.

The fresh standalone build and attempt are retained at
`output/diagnostics/native-pole-dual-shared-small-build-1` and
`output/diagnostics/native-pole-dual-shared-small-1`. The attempt binds 21
immutable files, including exactly one archived shared core, the accepted
baseline result, compiler/link inputs and fixed native dependency snapshots.
Its probe SHA-256 is
`289c809587e3ba04ce55c4d97bb488d10884c17533ee3803516ac2079a3e4ed2`.

The process exited zero without timeout in 7.688539304 seconds, with peak RSS
9,216 KiB, under the unchanged 180-second limit, five-second grace and 30-GiB
address-space cap on CPU8. These are diagnostic timings. All pre/post hashes
passed and the process was reaped.

All six cases and all 40 full coefficient rows match the original report,
including every retained 512/1024-bit value and comparison guard, native
coverage, component layout, tiny-leading relative checks and nonzero imaginary
control. Seven domain rejections, two source-bound rejections and the separate
literal-zero control also match. The independent audit compared all semantic
JSON fields after removing only timing fields and found no difference; it
retained `independent-review.json`, `independent-semantic-equivalence.txt` and postflight
hash evidence beside the attempt. This permits reuse of the shared helper in
the separately reviewed original-input adapter. It supplies no actual-input
oracle result or reader acceptance.
