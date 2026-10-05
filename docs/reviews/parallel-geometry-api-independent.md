# Independent review of the staged geometry work API

Read the public-interface proposal, ignored Rust sketch and existing native
`decompose`, support, cone, triangulation and monitor paths. This is a source-only
design review; placeholder types are not implementation, thread-safety or
performance evidence. The API has appropriate owners and introduces no
alternative geometry algorithm, executor, CAS or persisted work format.

Private retained `Arc` origins distinguish equal-input plans and separate
preparation instances without a global ID service. Typed chart/cone job IDs
identify ordinals, not scientific sectors. Private completion construction and
stage checks prevent arbitrary maps, wrong-stage payloads and foreign work from
being accepted. Exact ordered coverage, including duplicate/missing checks,
must precede returning a finished decomposition.

The proposed shared native seams preserve fixed-axis support restriction,
candidate order, one facet computation per prepared chart, cone-local pulling
cache, signed map exponents/determinants, Jacobians and positional valuations.
Canonical merge order avoids completion-order-dependent sector numbering.
Deferred chart errors correctly allow earlier cone failures to retain their
serial precedence. A valid zero-dimensional chart contributes its private
native trivial map with no cone jobs; an omitted stage is not implicitly empty
work. The serial path should keep its current chart-by-chart lifetime and
observer behavior rather than being forced through collect-all preparation.

Cancellation and progress are appropriately caller-owned. Worker callbacks must
describe only local progress; only successful final merge may emit global
Complete, and its callback can still cancel. A caller may retain error-bearing
completions, but no partial Decomposition is returned or admitted to the cache.
More facets and completed out-of-order maps can remain live than in serial
execution; neither worker count nor cache entry capacity is a byte bound.

One failure-precedence detail needs explicit handling during seam design.
Current serial code obtains a full `pulling` result before converting maps, then
validates and appends one map at a time and checks cumulative `max_sectors` after
each append. If a parallel cone completion retains only a terminal conversion
error and drops its successful map prefix, `finish` cannot determine whether
an earlier prefix would already have exceeded the cumulative sector limit.
The completion should retain a private prefix/error position, or use an
equivalent native seam, so a later conversion failure cannot mask an earlier
canonical global-limit failure. Pulling failures occur before map conversion
in the original path and should retain that distinction. This requires no
public partial-map API or new arithmetic. The author accepted the finding and
updated both proposal and ignored skeleton to retain that private prefix plus
terminal error. The revised skeleton SHA256 is
`529750f26a6313717c8990bfb9f4e4b3000d2de3f617140b592b6103426f5cf1`.
Its contract merges and checks the prefix before resolving the later failure;
pulling errors have no prefix. A focused native-seam regression is prescribed.
This resolves the design finding; actual implementation remains a separate
review and test gate.

The proposed tests are otherwise well scoped: exact serial equality under
reversed completion order and all domains, independent measure/valuation
controls, non-simplicial and signed infinity maps, zero-dimensional cases,
foreign/stage/duplicate/missing work, native resource limits and cancellation at
each boundary. Real payload `Send + Sync` assertions and caller-scoped threads
are necessary before implementation acceptance. Existing serial/cache/context
tests remain required. No source-level review establishes a speedup.

## Concrete implementation review

Independently read `stages.rs`, `work/{mod,job,assembly}.rs`, the serial
`decompose.rs` extraction and both public/private test modules. No additional
source-level blocker was found. Shared native chart preparation and candidate
decomposition retain the existing arithmetic, pulling and map-validation owners;
serial execution still completes one chart before preparing the next. The only
new count arithmetic is checked metadata accumulation, not geometry.

The actual private `Arc` owner variants bind chart completions to one plan and
cone completions to one prepared stage. Admission rejects missing, duplicate,
foreign, wrong-stage and unknown jobs before any payload can produce a fan.
Prepared offsets enumerate only successful charts before the first canonical
preparation error. Canonical final merge returns earlier native cone/limit
errors before that deferred chart error. Zero-dimensional charts use the native
trivial map with no cone jobs and the original sector-limit check.

The resolved precedence finding is implemented: cone completion retains native
map prefixes and a terminal error privately. Final merge appends/checks each
prefix map against the cumulative limit before returning the later conversion
error. A native pulling failure precedes conversion and retains an empty prefix.
The private fault tests exercise both cases, as well as an earlier chart's cone
limit versus a later chart preparation error. They do not expose a public fault
constructor or fabricated decomposition.

The public tests cover ordered exact maps and counts in all three domains under
one/two caller-owned threads and reversed arrivals; a rational cube moment1/6;
positional translated valuations; the signed infinity chart identity; all three
native limits and invalid inputs; stage/coverage admission; cancellation during
work, merge and the final observer; and zero-dimensional progress. Actual opaque
payloads have compile-time `Send + Sync` checks. No pool, Atom state, persisted
work schema or cache insertion API was added. The additional private facet-counter
control verifies three projective chart preparations occur once and no further
facet preparation occurs across their candidate jobs.

The author's final sector gate passes38 tests with2 ignored, including all28
existing tests, and sector all-target Clippy passes with warnings denied
(`output/parallel-geometry-final-tests.log`, `output/parallel-geometry-clippy.log`).
Independently executed the six public tests and four private work tests from the
compiled numeric-only binaries: all10 pass, with no Symbolica initialization.
Logs are `output/parallel-geometry-independent-tests.log` and
`output/parallel-geometry-independent-private-tests.log`. This closes the
implementation review with no remaining finding. Main-workspace integration
remains the coordinator's separate gate; no speedup or production scheduling
adoption is claimed by these sector-library checks.
