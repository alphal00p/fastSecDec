# Generation parallelism and memory review

Date: 2026-10-07. Independent review of the symmetry and memory implementations;
the reviewer authored the separate basename-preflight change.

The symmetry change preserves native ownership. Each chart's complete factored
density is assembled with Symbolica `Atom` operations, including prefactors,
regulator powers and numerators. The existing incidence encoding still delegates
canonization to Graphica. Its `CanonicalForm` and native density move through an
opaque completion; neither an alternate graph canonicalizer nor an approximate
expression comparison was introduced.

Each dispatch has a fresh owner token. Admission rejects foreign owners, wrong
stages, incomplete coverage and duplicate or omitted indices, then orders all
completions by their original chart index. Representative registration remains
serial in that order. A canonical graph match is still only a candidate: native
simultaneous literal substitution must reproduce the complete representative
density exactly. Thus scheduling cannot select a different representative or
discard a distinct numerator, regulator dependence or weight. The core creates
no pool; the existing bounded CLI executor owns scheduling and worker activity.

Preparation is timed once by elapsed dispatch wall time; ordered verification
adds its own measured time. Worker completion counts describe preparation;
coordinator counts describe subsequent exact comparison. The UI now names those
subphases explicitly. Review found that the former coordinator ETA reused total
symmetry time, which would charge the new parallel preparation cost again. This
was corrected: only a matching live worker workload supplies an ETA; coordinator
work reports it unavailable. Cancellation remains cooperative at native job
boundaries, not preemption inside graph canonization. Prepared chart completions
are retained until ordered admission; there is no new hard memory cap.

Memory sampling reuses `sysinfo` 0.39.6 with only its `system` feature. Source
inspection confirms that `Process::memory()` reports resident bytes (on macOS,
`pti_resident_size`). The monitor refreshes only the current PID with task scans
disabled, plus native system RAM counters. It runs on existing dashboard polls,
at most once per 500 ms, and creates no polling thread or subprocess. Unavailable
measurements are optional values. The observed peak is explicitly a maximum of
samples, not the OS lifetime high-water mark. System used, total and available
retain OS accounting; available is not fabricated as total minus used.

The monitor and dependency belong exclusively to the CLI. An independent
`cargo tree -i sysinfo` check shows only `fastsecdec-cli` as a dependent. RSS is
reported for the entire process, never divided among worker threads. Nothing is
added to native numerical status, evaluator state or portable core dependencies.

An independent live CLI probe generated a small direct integral and checked all
27 JSON status snapshots: positive process RSS, sampled peak at least current
RSS, sensible system totals, 500 ms cadence metadata, and no per-worker memory
claims. Its symmetry snapshots show preparation workload followed by ordered
comparison with the workload cleared. The observed initial RSS was 9,207,808
bytes; total system RAM was 38,654,705,664 bytes. These are a sampler check, not
performance measurements. Evidence is under ignored
`output/generation-preflight/memory-status.jsonl`.

No blocking source finding remains. Subsequent focused probes compare serial,
one-worker, four-worker and reversed-completion generation for symmetric,
asymmetric and opposite-sign numerators. Coefficients, representative charts,
permutations, kernel mappings and compiled identities agree. Both native and
portable eager implementations pass; malformed completion ownership/coverage
and cancellation are checked separately. See [parallel symmetry](parallel-symmetry.md).

The root's corrected D05 comparison uses the preceding specialized-model input
to isolate this scheduling change. Its eight-worker artifact has the same
semantic kernel identity and byte-identical `.dat` payload as the preceding
serial artifact. Symmetry takes 8.024 s (8.001 s preparation, 0.022 s ordered
comparison), with eight simultaneously busy workers, compared with the prior
39.581 s stage measurement. Whole generation takes 46.200 s and the observed
sampled peak RSS is 3,244,425,216 bytes. Other stage timings also changed, so this
is not a controlled whole-generation speedup claim. Evidence is under ignored
`output/generation-followup/`; the subsequent runtime-model schema revision
requires its own validation and is not covered by this byte-identity result.

No persisted tests or other examples were changed by this review. Existing
one-loop master/reduction reference APIs are unaffected; the symmetry boundary
is checked by exact native expression equality rather than numerical references.

The host has Rust 1.99 but no `nix-shell`; these checks use the installed native
toolchain. Existing symmetry unit tests currently fail to compile because six
deferred test-only `SectorProgram` initializers omit the earlier runtime input
field. Focused executable probes supply the evidence for this revision without
silently migrating the user's deferred tests or claiming their gates passed.
