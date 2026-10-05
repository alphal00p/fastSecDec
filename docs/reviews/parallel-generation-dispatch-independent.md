# Independent review: cache-aware geometry dispatch proposal

Read `parallel-generation-dispatch-proposal.md` and the disconnected signature
sketch `output/probes/parallel_generation_dispatch_api.rs` against the reviewed
native work API, geometry cache and generation-context contracts. No design
blocker was found. This is a source-only interface review; no implementation,
main-library runtime or speedup claim is implied.

The dispatcher receives native jobs and can return only private native
completions. The cache itself creates each plan and runs its stage admission and
canonical merge, so this seam cannot inject a caller-created fan. Returning
missing, duplicate, foreign or wrong-stage work stays a typed native error.
A dispatcher error represents caller scheduling failure; callers retain native
error-bearing completions when they need canonical scientific error precedence.
The callback owns its threads, observation channels and joins. No new executor,
geometry implementation or expression state is introduced.

Cache hits correctly avoid dispatch. A successful cold request runs the chart
and cone stages, including an explicit empty cone stage where appropriate.
Only complete native merge accepted by the final observer can enter the cache;
dispatch, worker, coverage, limit or cancellation failures cannot evict valid
entries or insert partial geometry. The existing zero-capacity and exact ordered
support/domain/limit key semantics remain intact. A later generation-event
cancellation may retain already accepted geometry, as in the current context,
but must not return a completed integral.

The main entry keeps domain assessment and support extraction before lookup and
all mapped residual, numerator/weight, subtraction and Laurent work afterward.
Thus warm geometry reuse cannot become a positivity or scientific-equivalence
certificate for different coefficient signs or weights. Existing methods and
events remain unchanged. Immutable cancellation through a caller token can be
checked by the phase wrapper and native stages without sharing a mutable global
observer among workers; local job progress remains distinct from accepted map
counts. Cooperative native arithmetic boundaries still govern latency.

The proposed acceptance tests cover the actual seam: two caller-owned workers
with reversed arrivals, all coefficients/exact offsets/map metadata versus the
serial path and an analytic control, cold/warm dispatch counts, changed regular
weights, same-support domain rejection, zero-dimensional and empty inputs,
invalid completions and cancellation before/after accepted cache insertion.
Retain existing serial/cache/context tests. These can establish reusable parallel
scheduling; retained memory and speedup require separate measurements afterward.
