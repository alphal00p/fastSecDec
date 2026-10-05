# Private native request seam — independent review

The source-only [request interface](native-named-request-interface.md) is
accepted on 2026-10-05, subject to implementation and focused scientific/error
controls. It separates the parent-owned native Series/retry/composition logic
from one attempt's local request registry and native alias lowering. Four
private operations (`new`, `wrap_series`, `check_scalar`, consuming `lower`)
are sufficient; no public symbolic container, worker pool or generic task
framework is required.

The parent `generation/coefficient_first.rs` owns the native Series alias and
shared limit/count/progress/result types. Its request child owns exact body
deduplication, strict native symbol admission, separate unsubstituted-partial
and full-request caches, and flat native `AliasedAtom` definitions. Interleaved
faces must enter only the full-request cache and remain downstream of existing
regularity admission. Final vectors rejoin the existing native program builder
with one shared map, preserving signed keys and literal zero entries.

The proposed unique-request limit is precise: count a full native tuple of
source, depths and argument Atoms once, check capacity/overflow before its
expensive resolution, and commit completed counts/cache entries only after
success. A cache hit consumes no new unit. The cap does not bound the number or
size of named source coefficients, native intermediate memory or derivative
steps; those distinctions must remain in caller documentation. A zero budget
can still admit outputs with no owned request.

Native source confirms the callback boundary. `Series::map_coeff` accepts `Fn`,
maps the coefficients, then performs native truncation. `AtomCore::replace_map`
accepts `FnMut`; an unset `Settable` keeps a subexpression unchanged, and native
normalization can still run after the traversal. Neither inspected API exposes
a fallible callback. A local first-error latch, with temporary `RefCell` only
where the `Fn` signature requires it, adapts these APIs without a new Series
constructor or replacement engine.

Implementation must preserve these failure rules:

- Once an error or cancellation is latched, perform no further request work
  or observer calls. Return the original coefficient from subsequent
  `map_coeff` callbacks and leave subsequent replacement outputs unset.
- Inspect the latch immediately after the native call, before coverage,
  unresolved-name checks, further cache commits or final alias assembly.
  Return the original error, never a later consequence of partial naming.
- Discard the complete attempt registry and candidate after any such failure.
  The composition owner must not resume a failed registry; a separately valid
  width retry starts with a new registry.
- Native traversal and normalization remain nonpreemptible. The latch promises
  no further adapter work, not immediate interruption of a native CAS call.

The borrowed `FnMut(RequestProgress) -> ControlFlow<()>` fits the existing
caller-driven generation observer. It is not stored, shared with workers or
called after the first failure. Counts describe completed request work, not
physical sectors or subtraction contributions. Only the outer generation
owner may emit integral completion and own phase timing.

The accepted native symbol-hygiene composite remains mandatory before any
formal Atom is constructed. Owned malformed arity/depth/unresolved requests
must use typed errors; unrelated functions remain native evaluator input.
Neither process-state conflicts nor resource/cancellation errors trigger a
physical fallback. Native exact zero-slope admission remains the separate
composition owner's physical-fallback decision.

No source-level blocker remains. Required implementation controls include the
existing exact full-vector/composed-face/cache tests, request-cap boundaries,
cancellation at each native step, preservation of the first error and absence
of usable partial output, plus real allocator hygiene/repeated-job tests.
Public opt-in generation, complete original-oracle comparison, cold replay and
whole-graph acceptance remain separate gates; this review executes no science.

The subsequent concrete request implementation follows this seam: native
symbol lookup and strict metadata/hook admission precede Atom construction,
the request budget is checked before resolution, and the two infallible native
callback adapters inspect their first-error latch before any later validation.
The original resolver caches only unsubstituted partials; restricted
interleaved results enter the full-request cache. Lowering consumes the
registry and gives every coefficient the same complete native alias map.
No implementation source blocker was found in these request modules.

The new request tests cover native bounds and body deduplication, mixed and
composed derivatives, early faces and exact zero, budget boundaries, malformed
requests, cancellation, actual allocator hygiene and two live local vectors.
Review caught one test-isolation issue: a process-global interned-name count
could change because another test created a name. The author replaced that
assertion with this job's source/alias handles and local counts; the earlier
isolated hygiene process remains the namespace-growth evidence. These are
source-review findings, not a report that the new tests have executed.

The composition source retains the accepted native arithmetic sequence and
checks the actual absolute remainder after every attempted width. Empty input
uses native relative-depth zero; no zero shortcut is needed. In particular,
the earlier absolute-truncation zero control must not be generalized to claim
that relative zero loses its coverage. Unregulated axes still reach the
existing exact physical admission before native named composition. Final
controller tests, a coherent compiled snapshot and focused executed gates are
pending before accepting this private implementation slice.

The next public seam is also accepted as a design: one native
`CoefficientExpansionOptions` value selects default `Physical` or opt-in
`NativeNamed` and carries optional attempt, relative-width and unique-request
caps. The CLI may deserialize that pure-data native type directly under
`generation.coefficient_expansion`; it must not define a second policy. Those
caps govern native named composition, while the existing physical route and
exact-unregulated fallback retain their existing subtraction admission.

Typed progress distinguishes requested named expansion from effective physical
fallback and labels counts as belonging to the current attempt. A named
representative uses one exclusive `CoefficientExpansion` duration, including
fallback work, without also recording that interval as physical subtraction
or Laurent time. An optional typed snapshot is absent on unchanged physical
runs; absent historical snapshot/timing fields must read as `None`/zero. During
an active representative the completed count excludes it, and the terminal
representative event increments that count. Presentation may reuse the existing
caller cadence, forcing first/attempt/fallback/completed/final boundaries, but
must poll terminal cancellation even when it suppresses rendering.

These are wiring requirements, not an executed public-entry acceptance. The
fresh conditioning basis remains an explicit observation; loading the unchanged
native artifact must not invent a physical or named basis from stored rows.

## Private executed gate

The private implementation subsequently passes all 14 focused tests: six
controller/composition tests and eight request tests, with no failures or
ignored tests. The controller matrix compares the complete signed union with
physical Taylor and IBP across Gamma poles, complex values, negative requested
maxima and fractional endpoint powers; every expected-nonzero matrix case has
an explicit nonempty guard. Separate analytic double-pole and fallback values,
exact cancellation, native zero/empty coverage, unknown remainders, typed
errors, all caller caps and cancellation categories are checked. The request
tests execute the local ownership, hygiene, fallback and first-error controls
described above.

`output/diagnostics/coefficient-first-private-1` records exit zero without a
timeout in 0.083535162 seconds, with 37,028 KiB peak RSS, under the prescribed
180+5-second/30-GiB address-space/CPU-8 bound. The same executable then passes
15 existing subtraction/Series controls, with two explicit replay probes
ignored, in `coefficient-first-physical-regressions-1` (0.145819072 seconds).
Both process records retain unchanged executable/input/build-evidence checks.
The independent review matched all test names to the reviewed sources and
the tested executable to the Cargo JSON artifact; source timestamps precede
that build. Both no-run builds succeeded, in 17.81 and 6.51 seconds. The second
adds the nonempty guard; there was no failed build in this slice.

The stable eight-file source snapshot and digest record are retained in
`output/diagnostics/coefficient-first-private-source-1`. The independent review
verified all eight hashes and matched the archived controller/composition/
request sources to the reviewed files. Package formatting and scoped main
library/test Clippy pass (11.68 seconds); the existing native Symbolica unused
`Result` warning is unchanged.

These focused scientific results accept the private implementation boundary.
Public options/coordinator/status adoption, full workspace checks, public cold
replay and actual full-graph
acceptance remain separate gates. This result does not change the default
physical route or certify a speedup.

## Public opt-in source and focused gate

The concrete public coordinator and CLI/status bridge have no remaining source
finding. `Physical` remains the default. `NativeNamed` starts from the already
admitted mapped terms before constructing a physical subtraction Atom, uses the
reviewed native composition/request owners, and rejoins common assembly before
the single multiplicity application. Existing signed-vector padding, exact
offset extraction and chart-to-kernel association remain in that common path.
The exact unregulated-endpoint fallback retains the physical pruning and error
owner; it is observed as a fallback and never used to conceal a resource or
native Series failure.

Fresh named sectors record `MappedEndpointBound`: the componentwise maximum of
the native subtraction counts over every mapped term. Each retained physical
remainder row is componentwise bounded by this row. On the open unit cube the
nonnegative coordinate log weights therefore make the existing lost-bits
heuristic at least as conservative. This is a precision-check scheduling bound,
not a floating-point error certificate. Physical and fallback sectors retain
the actual remainder basis. Only rows and checked total degree enter the
existing numerical artifact; cold loading does not invent a basis label.

The independent review verified all 16 files in
`output/diagnostics/coefficient-first-public-source-1/source.sha256` and their
equality to the handed-off source. Seven bounded process records in
`coefficient-first-public-focused-1` report 40 passing tests, zero failures and
no timeout, with executable, input and build-evidence checks unchanged. The
subsequent eight-test public target rerun adds one admitted-chart cancellation
control, giving **41 distinct focused tests**. The latter retains a real chart
with no kernel and exact zero offsets, rather than treating empty input as
evidence of chart preservation.

The public tests compare every signed coefficient and exact offset, with
explicit output-length assertions, across Taylor/IBP, Gamma poles and negative
or positive requested maxima. They check geometry, symmetry multiplicity,
metadata, conservative row domination, cold/warm/dispatched geometry reuse,
changed local coefficient bodies, successful and rejected unregulated
fallbacks, all named caps, and cancellation without integral completion.
The extended alias/Gamma tests exercise hidden complex images, an independent
native Series/512-bit reference, public compilation and cold version-3 load,
worker clones and weighted replay with both ordinary and increased minimum
precision. A separate writer/cold-reader process test now covers both methods.
The unchanged artifact safety and caller-context tests also pass.

The first public no-run build failed on two test API uses (`GeometryJob::run`
returns an opaque completion, and an Atom predicate needs a value closure).
The corrected build and all scientific runs passed. The first scoped Clippy
run found two collapsible test `if` statements; the final equivalent let-chain
snapshot passes formatting and scoped library/test Clippy in 10.75 seconds.
Those style-only changes will also execute in the combined workspace gate.
All failed build/lint records are retained separately from scientific outcomes.

The bridge directly deserializes the native pure-data options and keeps
requested/effective method, per-attempt counts and exclusive timing explicit.
Named coefficient JSON presentation reuses the existing cadence, while every
callback still polls cancellation. Other generation JSON and the existing
terminal/plain intervals retain their prior behavior. The two CLI process
tests subsequently pass in `output/coefficient-first-cli-focused-2.log`: the
first failed resource-error assertion had read stderr instead of the existing
JSON error object on stdout and was corrected only in the test.

The first combined workspace run then exposed a genuine presentation
regression: applying the cadence to all generation JSON could suppress a fast
chart-to-cone dispatch transition. The accepted source fix limits coalescing to
the new coefficient stage, restores all previous generation JSON and
terminal/plain behavior, and leaves the geometry acceptance test unchanged.
The corrected targeted rerun passes five tests (two coefficient, two geometry
and one cadence control). The second full workspace run,
`output/coefficient-first-workspace-tests-2.log`, independently totals **370
passed, zero failed and 23 ignored across 62 summaries**. It executes the final
test style changes, all new public and bridge controls and the existing HEPKit
master/reduction, integration, covariance, persistence and geometry suites.
Formatting passes; workspace all-target Clippy finishes successfully in 11.81
seconds. The first wrong-stream assertion and first full cadence-regression
logs remain intact. This closes public opt-in adoption and its CLI bridge,
without changing the default physical method. The reconstructed actual public
representative remains a separate gate; no whole-graph completion or
performance claim follows from this acceptance.
