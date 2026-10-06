# Parallel sector-equivalence preparation (2026-10-07)

The generation stage now displayed as **Finding equivalent sectors** prepares
each mapped sector independently on the caller's existing generation executor.
The previous implementation assembled its complete symbolic density, encoded
the incidence graph and ran native graph canonization inside a serial registry
call. These operations do not depend on earlier representatives. Only the
subsequent exact comparison and deterministic representative selection depend
on the registry.

## Native ownership and API evidence

`generation/symmetry.rs` still assembles the same full mapped density, including
prefactors, numerator terms, Jacobian and regulator-dependent powers. Its
existing expression-tree encoder preserves shared integration-variable
vertices and treats external scalar expressions as native Symbolica values.
Graph equality proposes a candidate; the unchanged native simultaneous
substitution check must reproduce the complete representative density exactly
before any merge. This is sector equivalence, not threshold certification.

Graphica 3.0.1 owns `Graph::canonize` and `CanonicalForm` (`src/lib.rs`, public
definitions at lines 1383 and 1367 in the resolved source). The source and owner
tests were inspected before the split. They provide the required exact graph
operation; no new graph canonicalizer, physical graph type, algebraic hash or
numerical equality test is needed. The orchestration missing from the old
FastSecDec path was independent scheduling of these existing operations.

`PreparedDensity` holds the native density and canonical form. They move from
the job into the ordered registry, without cloning the large density or graph.
`SymbolicStage::Symmetry` uses the existing opaque `SymbolicJob`/completion
interface. The library creates no threads or pool. The CLI's bounded Rayon pool
executes mapping, preparation, coefficient work and compilation; portable
callers can retain the serial entry or supply their own dispatcher.

Admission verifies the fresh call owner, homogeneous stage, exact completion
count and consecutive source indices, then sorts by source index. The earliest
verified source remains the representative. Completion order cannot change
chart numbering, permutations, multiplicity or kernel order. Existing errors
remain errors; no incomplete generated integral is returned. Cancellation is
checked before density assembly, before and after native canonization, and
around ordered admission. A native algebra/canonization call itself remains
nonpreemptible. Existing one-loop masters, tensor reduction, evaluator and
integration providers are unchanged; this slice adds no numerical algorithm.

## Presentation and timing

Workers report real preparation activity under **Finding equivalent sectors**.
Aggregate details say **Preparing comparisons**; ordered registration says
**exact comparison** and clears the worker workload. Thus the latter neither
inherits idle worker rows nor reuses preparation throughput for a false ETA.
The shared `GenerationStage::label` supplies human labels to terminal and plain
status. Current-stage progress and ETA remain distinct from total elapsed time.

`symmetry_seconds` contains preparation wall time plus ordered admission wall
time, never the sum of concurrent worker durations. The stage timing emitted
after preparation and before its first ordered comparison exposes that split.
Coefficient activity now says **epsilon expansion pass N (relative depth W)**.
This depth is native `SeriesDepth::relative`: it is measured from each series'
leading epsilon power, not the final integral's requested order or a count of
nonzero terms. Admission before a pass no longer displays pass zero.

Configuration/status names are `coefficient_series` for the shared regular
coefficient-series composition and `full_expression` for expansion of the
complete subtracted expression. The default algorithm is unchanged. The older
`native_named` and `physical` strings remain input aliases; Rust enum variants
remain unchanged. Python defaults/parsing/status and the ggHH card/exporter use
the descriptive names. No other example or existing test was migrated.

## Focused correctness and measurements

A temporary public-API probe ran on both the native and portable host backends:

- The symmetric three-variable denominator `x*y+y*z+x*z` produced six charts
  and one representative. The numerator `x+2*y+3*z` and the opposite-sign
  numerator `x-y` each retained six representatives.
- Plain serial generation and caller-dispatched one/four-worker generation,
  including reversed completion order, agreed exactly on all coefficient
  Atoms, chart representatives, permutations, kernel indices, maps,
  conditioning rows and compiled kernel content identities. This also checks
  multiplicity in the resulting coefficients.
- Cancellation after density assembly returned `Cancelled`; omitted symmetry
  work, a foreign mapping completion and a foreign symmetry completion were
  all rejected. Opaque completions cannot be cloned or forged by safe callers.
- Canonical method strings serialize, and both retained aliases parse to the
  same existing algorithms. The eager portable evaluator remains usable.

Separate native CLI processes using one and four workers produced byte-identical
binary artifacts for the symmetric control through both `full_expression`
(3,649 bytes) and `coefficient_series` (3,679 bytes).

The root's isolated release D05 run observed eight simultaneous symmetry
workers. Preparation took **8.001481083 s** and ordered admission took
**0.022137375 s**, for **8.023618458 s** total symmetry time, compared with
**39.58 s** in the retained preceding serial-preparation run. All 30 charts had
distinct representative keys. The kernel content identity and entire binary
artifact were identical to that preceding D05 artifact. Generation metadata
reported 46.200 s overall (46.222 s through saving), and the memory sampler
observed a process RSS peak of 3,244,425,216 bytes. These are local observations;
other stage timings differed between runs, so they do not establish a matched
whole-generation benchmark or a memory bound.

Native core/CLI strict Clippy and the CLI build passed. Attempting the existing
symmetry unit-test target failed during compilation of six unrelated test-only
`SectorProgram` constructors that still omit the new runtime-parameter field.
Those test migrations remain deferred as requested; no unit-test pass is
claimed. Temporary probe sources, status and binary output are retained only
under ignored `output/generation-followup/symmetry-probe/` after execution.

## Remaining limits

All mapped charts were already retained by parallel mapping. The new stage
additionally retains prepared graph/density results until ordered admission;
duplicate-heavy inputs can temporarily retain candidates that are then
discarded. Job concurrency is bounded, but the result barrier holds one result
per chart. No general memory bound or streaming scheduler is claimed.

Exact candidate comparison remains on the coordinator. The D05 measurement
shows that remainder is small for its distinct canonical keys, while the merged
control checks its correctness. It is not a performance result for a large
duplicate-heavy registry. Parallel canonical groups could be considered if a
future measurement identifies that exact comparison as a bottleneck. Native
graph canonization itself was not modified, and no new actual Pyodide/Wasm wheel
or browser performance result is claimed.
