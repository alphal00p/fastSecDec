# Independent review of caller-owned exact geometry reuse

Reviewed the new sector-only `GeometryCache` API, native support/domain and
option definitions, current decomposition/completion path, and six public
regressions. No actionable correctness or ecosystem-reuse defect was found.
There is no Symbolica dependency, changed decomposition algorithm, alternate
algebra, background execution or persisted cache format in this slice.

The key contains the complete ordered `PolynomialSupport` values, domain, and
all three existing resource limits. Native support construction already sorts
and deduplicates monomial rows; exact equality retains coordinate order and
monomial translations. The cache introduces no further normalization. Reordered
factor supports therefore cannot reuse positional valuations from another
request, and the same fan with translated supports cannot reuse old valuations.
A changed domain or limit likewise cannot bypass native validation or a tighter
resource bound through a warm entry.

Misses call the unchanged `decompose` and forward callbacks. Insertion and FIFO
eviction happen only after success and acceptance of native `Complete`, so
errors and cancellation—including the final completion callback—preserve the
old entries. Hits emit the recorded `Complete` callback once and remain
cancellable without deleting the valid entry. Its status describes the cached
result, not new work; the explicit `reused` field and API documentation preserve
that distinction. `Arc` shares only completed exact geometry, and eviction or
clear does not invalidate outstanding callers. Capacity zero retains nothing;
the documented capacity is an entry count, not a memory bound.

The tests exercise domain separation, native equality of all returned maps,
canonical support equality, coordinate/factor ordering, translated valuations,
all resource limits, invalid input, final and hit cancellation, FIFO/disabled
retention and caller-owned scoped threads. Their independent exact cube moment
`integral x^2*y = 1/6` checks the mapped measure alongside native map equality.
No additional test that mirrors the cache implementation is needed.

The author reports all 28 sector tests passing with two explicitly ignored
probes and clean sector all-target Clippy. Independently executed the already
built `geometry_cache-52c101fbc1e22490` target: **six tests passed**, serially,
without Cargo or Symbolica initialization. Evidence is
`output/geometry-cache-independent-tests.log`. Main-library/CLI adoption and
cache performance are separate future slices; this review does not claim an
end-to-end speedup.
