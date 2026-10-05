# Complete geometry cache: first slice

Author implementation record, 2026-10-05. This implements only the first slice
of `geometry-reuse-scheduling-proposal.md`; independent review is tracked
separately. The production release/source archive at `6332676` was frozen before
these edits, and the ongoing full-graph diagnostic uses that immutable snapshot.

`fastsecdec-sectors::GeometryCache` wraps the existing `decompose` function.
Neither the decomposition algorithm nor the default cache-free entry changed.
`GeometryCacheOutcome` returns an `Arc<Decomposition>` and an explicit `reused`
flag. It retains only complete native results: no public insertion method can
admit a collection of maps whose fan coverage has not been established.

Keys use exact native equality for the domain and ordered polynomial supports,
plus all three existing resource limits. Each support already carries its
dimension and canonical exponent rows. Coordinate and factor order, repeated
factors, and monomial translations remain significant. There is no expression
parser, hash-only lookup, alternative arithmetic, CAS operation or persistence
schema. Native domain assessment of a future integral remains outside this
geometry cache.

Capacity is a maximum **entry count**, not a byte bound. Insertion order gives
deterministic FIFO eviction; hits do not change that order. A zero capacity
disables retention. `clear` releases cache ownership, while outstanding returned
results remain valid. Lookup is a bounded linear native-equality scan, avoiding
a second copied key index; no speedup or memory-size claim is made without
measurement.

Misses forward native progress and cancellation unchanged. A result enters the
cache only after native completion and acceptance of its final callback. Hits
invoke the callback once with the retained native `Complete` report: the report
describes completed geometry, not stages performed during this lookup. A
cancelled hit leaves the prior good entry intact. Any failed or cancelled miss,
including rejection at final completion, neither inserts nor evicts an entry.
This API owns no scheduler, global state, thread pool or cancellation token.

Validation completed without Symbolica initialization:

- `cargo test --locked -p fastsecdec-sectors`: **28 passed, 0 failed, 2 ignored**
  expensive geometry probes. Log: `output/geometry-cache-tests.log`.
- The six new tests cover exact equality with serial native maps across domains
  and zero dimensions; independent cube moment `integral x^2 y = 1/6`; factor
  order and translated valuations; all resource limits; early/final/hit
  cancellation; failed-request non-insertion; bounded retention and result
  lifetime. Compile-time `Send + Sync` assertions and caller-owned scoped
  threads prove native geometry can be shared without a library executor.
- `cargo clippy --locked -p fastsecdec-sectors --all-targets -- -D warnings`
  passed. Log: `output/geometry-cache-clippy.log`. Changed Rust files were
  formatted and `git diff --check` passed.

Generation-level context wiring, progress integration, parallel chart/cone
dispatch and additive per-sector content IDs are deliberately separate tasks.
This gate is not evidence of full-graph performance or cached-generation
scientific equivalence, which require the later integration boundary.
