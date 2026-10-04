# Independent native Laurent capture/replay review

This read-only review covers the test-only files under
`generation/laurent/profiling` and their two `cfg(test)` hooks. The reviewer did
not author the diagnostic. It investigates the bounded on-shell triple-box
generation timeout; it is not a production series replacement.

The capture hook runs after the existing native placeholder-template construction
and before native series expansion. It exports the original expression, exact
template, regulator and coordinate images using Symbolica's native Atom binary
transport. The representative traversal index, source chart and multiplicity
are explicit; multiplicity is applied later in ordinary generation and is not
silently included in the captured template. Earlier representatives are skipped
only inside the test-only capture scope. Domain checks, geometry, full-density
symmetry and subtraction still use their ordinary native implementations.

The scope is thread-local and its RAII guard clears on ordinary return or unwind.
Nested capture scopes fail immediately. A successful target capture returns
`GenerationError::Cancelled` instead of a generated integral. The initial source
had one fail-closed gap: an out-of-range target could skip every expansion and
allow ordinary generation to return an empty result before the ignored caller
asserted failure. The author added a `cfg(test)` active-capture barrier before
the generation `Complete` event/return, with a missing-target control. The
control captures a later representative after skipping an earlier one, rejects
an unreachable target, then verifies ordinary generation after the guard drops.
The safeguard finding is closed by source review; it concerns diagnostic
isolation, not ordinary production behavior.

The replay uses native `SeriesDepth::relative`, `get_trailing_exponent`,
`absolute_order` and `terms`. Symbolica's `derivative.rs` owns cancellation and
depth refinement: relative depth counts the retained width from the lowest
represented exponent, while absolute expansion includes the requested exponent.
The diagnostic first requests relative width one, then, if necessary, requests
`maximum + 1 - first_trailing_exponent`. It accepts a result only when the
returned native absolute order is strictly greater than the requested maximum.
Thus a misleading first bound becomes an explicit failed experiment; no hand-
written epsilon expansion or coefficient convolution is introduced.

Controls cover high Gamma/endpoint poles, cancellation before the first retained
coefficient, a negative maximum order, a monomial entirely above the requested
range and zero. The reviewer additionally requested an explicit fractional-power
control: native Puiseux representation may contain such powers, but FastSecDec's
integer Laurent-vector extraction must reject them. The added control requires
the same typed fractional-order rejection from both ordinary and replay paths.
Exact comparison between
absolute and relative replay imports native coefficient Atoms and checks every
retained order, both before and after coordinate-image restoration. Template
preparation and native restoration are separately measured; multiplicities,
evaluator construction and integration remain absent and must not be included
in any claimed replay speedup.

The reviewed execution log now records all three controls passing in 0.15 s,
including fail-closed capture/missing-target restoration and fractional rejection.
The bounded target capture passed in 6.173 s process wall time without timeout.
Its fixture reports 2,112 charts and 1,026 representatives at the original
on-shell point. It captures index 38 (displayed ordinal 39), source chart 46,
multiplicity two, maximum order zero and nine retained coordinates. The original
expression has 14,505,102 native Atom bytes; the opaque template has 5,736,028
bytes. All 201 coordinate-image/value-symbol pairs and three base Atom files are
present, and the archived reviewed-source SHA-256 records match. Template
preparation took 0.349 s. These measurements are retained under
`output/diagnostics/laurent-capture`.

Both bounded replays now completed. Absolute native series took 148.974 s and
relative native series 37.332 s (including 4.581 s for the first relative-width-
one request). The native first bound was −4 with trailing exponent −5; requested
width six reached absolute bound one. Both produce orders `[-5,-4,-3,-2,-1,0]`.
Separately measured restoration took 4.445/4.267 s. The comparison process passed
in 27.308 s, checking all six native coefficients both before and after image
restoration. The restored finite coefficient is still 256,030,180 Atom bytes in
both modes. These are one-representative attribution results; they do not show
whole-generation performance parity or resolve final expression size. Any future
production change requires the complete scientific gate and typed failure
handling, rather than promoting test assertions.

The first cold replay logs warned that imported Gamma lacked user-defined
functions. This prompted a narrow callback audit before accepting the timing.
Native import compares the symbol-table size before and after `SymbolBuilder`;
that builder can trigger global builtin initialization, so the warning alone
does not prove callbacks were lost. Existing callback definitions are preserved
when an imported definition supplies none. The reviewed cold control imports the
actual template before any symbolic construction, then invokes its imported
Gamma symbol directly: `Gamma(5)=24`, the derivative at one, and the series of
`Gamma(1+eps)` through order four all pass. A second process initializes Gamma
first and produces byte-identical canonical template/control expressions; the
archived SHA-256 records agree. This proves the warning is a lazy-initialization
false positive for this capture and validates the original paired comparison.
The later explicit native initialization is limited to diagnostic import entry
points and does not change the timed binary. Production kernel artifacts take
the separate native parsing
path, which initializes global state before parsing/normalization; the existing
fresh-process complex shifted-Gamma test through epsilon order four already
covers that path and subsequent worker evaluation. No production loader defect
or dependency patch is established by this diagnostic warning.
