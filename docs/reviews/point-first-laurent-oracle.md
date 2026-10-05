# Independent point-first Laurent oracle

This is a bounded diagnostic of the actual on-shell triple-box representative
at index 38 (display ordinal 39), source chart 46, multiplicity two. It is not
a production subtraction or integration implementation. The ignored source is
`output/probes/point_first_laurent.rs`; execution evidence will be retained
under `output/diagnostics/point-first-laurent/`.

The earlier comparison could check five pole coefficients but reached its
180-second bound while evaluating the very large finite coefficient. This
oracle reverses only the order of the native operations: it imports the
**original epsilon-dependent production-subtraction expression**, binds fixed
exact rational coordinates, then calls native Symbolica series. It does not
import a newly proposed compact expression, restore opacity templates, fit
values, interpolate Laurent coefficients, or expand a polynomial manually.

The original source is
`output/diagnostics/laurent-capture/index38/expression.atom`. Its native capture
contains the physical prefactor and the representative density before symmetry
multiplicity; the oracle likewise does not multiply by two. Native coordinate
exports come from the separate mapped capture because the earlier capture only
stored coordinate names. Before using them, the oracle checks byte-identical
`fixture.json`, equal representative/chart/multiplicity/order metadata, and each
imported coordinate's exact canonical name against the original capture. It
records BLAKE3 hashes of all imported source and metadata files.

The three prescribed points are:

1. `(1/2, 1/3, ..., 1/10)`;
2. `(1/10^20, 37/100, ..., 37/100)`;
3. alternating `1/10^8` and `73/100`, starting and ending with `1/10^8`.

These are exact rational points, not the binary64-rounded points used by the
earlier comparison. Any compact representation must be compared at the same
exact values. Native `replace_map` substitutes only matching literal variable
symbols; native `gamma()` is initialized before importing any Atom. The native
absolute series must have remainder order strictly above zero and integer
coefficient orders. For this particular fixture, all six nonzero coefficients
from minus five through zero are required, preventing vacuous zero-vector
agreement. Every coefficient is exported as a native Atom and evaluated with
the existing multiprecision evaluator at 512 and 1024 bits. The two values must
agree within `1e-70 * max(1, abs(value_1024))`; the full values and differences
are retained, not merely rounded binary64 summaries.

The coordinator approved a separate **180-second bound per point**. The shell
orchestrator `run_point_first_laurent.sh` reuses the reviewed Rust `process_timer`
and native `prlimit` with a **30-GiB virtual-address cap** (`RLIMIT_AS`), explicitly
not an RSS or aggregate process-tree cap. Each attempt uses a new output directory and records partial stage and
coefficient progress before the next native operation. All failures remain
evidence; the bound is not silently extended. Build evidence must identify the
exact source, compiler command, linked Symbolica/Numerica libraries and native
dependency patch state. The ordinary captured regulator is used unchanged. A
separate native fix for trailing-underscore regulator matching is being audited;
the oracle waits for that dependency rebuild and its explicit runtime handoff.

The standalone oracle now compiles against the reviewed native library. Build
evidence is `output/diagnostics/point-first-laurent-build/build-evidence.json`,
including the actual linker trace and SHA-256 hashes of the selected Rust
libraries. Symbolica's fixed opt-level-two debug library has SHA-256
`dda215a39b38358772677beed211e773c9b028f33f0251dc1072135fe37f2e51`;
the helper itself uses `rustc -O`. This profile is identified for correctness
reproducibility, not a timing comparison. Initial standalone compile diagnostics
(the macro's crate-name environment and an empty native evaluation map's key
type) are retained and resolved. The completed attempts below are bound to this
specific binary and dependency evidence.

Independent HEPKit/source review found no blocker in the literal coordinate
binding, capture association, six-order nonvacuity guard, native absolute
remainder check or multiprecision comparison. It also checked the production
generation ordering: symmetry multiplicity is applied after the captured
Laurent expansion, confirming the oracle's per-representative convention.
Pathfinder's second read-only review also checked that association and corrected
the draft Float magnitude calls to the existing native `Real::norm()` API before
compilation. The shell watchdog adapter was independently reviewed with no
blocking finding.

## Original absolute-depth outcomes

All three prescribed attempts finished without changing the points or bounds.
Point zero passed all six coefficients, including the finite term, with the
required 512/1024-bit agreement. Exact coordinate binding took 1.141 seconds,
native series 45.247 seconds and the complete process 55.402 seconds; peak
resident memory was 1,039,716 KiB. Its native remainder order is one. The six
native coefficient files and full-precision values are retained in
`output/diagnostics/point-first-laurent/point-0/`.

Point one reached its 180.151-second watchdog in native absolute series, with
peak resident memory 3,507,172 KiB. Point two likewise reached 180.213 seconds,
with peak resident memory 3,243,120 KiB. Neither produced Laurent coefficients;
their partial progress is evidence of an unresolved cost, not failed agreement
or a zero vector. The process reports, source hashes and final completion record
retain all three outcomes. These correctness attempts partly overlapped the
separate no-Symbolica external reference on another CPU; no performance claim
is made.

## Approved bounded relative-depth continuation

A separate source/binary and new output directory preserve the absolute attempts.
The continuation still binds the complete original captured expression at the
same exact rational points. Native relative depth one supplies the first actual
remainder and relative width. If its absolute remainder does not exceed zero,
the next native request is its relative width plus the exact deficit to absolute
remainder one. Checked integer conversion rejects fractional or unrepresentable
bounds. At most four native calls share the original 180-second per-point and
30-GiB virtual-address limits; no assumed leading pole or alternative series
implementation is introduced.

Point zero runs first and must reproduce all six successful absolute coefficient
Atoms exactly, as well as passing the unchanged multiprecision check. Only then
may point one run. Point two is excluded from this approved continuation and
would need a separate evidence-based decision. The ignored sources are
`output/probes/point_first_laurent_relative.rs` and its shell launcher. This is
an expansion-request diagnostic, not a change to the production algorithm.

The independently reviewed relative-depth continuation passed both approved
points. Point zero requested native widths one and six (native remainder bounds
minus four and one), reproduced every absolute coefficient Atom exactly, and
passed all multiprecision checks. Native calls took 2.131 and 12.596 seconds;
the process took 25.008 seconds with peak resident memory 350,808 KiB. Point one
also used widths one and six, taking 3.015 and 52.071 seconds. All six coefficients
passed 512/1024-bit agreement; the complete process took 69.136 seconds with peak
resident memory 1,103,804 KiB. Its failed absolute attempt remains unchanged.
The frozen source, actual linker trace and parent dependency hash checks are in
`output/diagnostics/point-first-laurent-relative-build/`; successful outputs and
process reports are in `output/diagnostics/point-first-laurent-relative/`.

Those two results justify the coordinator's separately approved replay of point
two using the same frozen binary, exact coordinates and unchanged bounds. This
final attempt also passed all six coefficients, including the finite part, at
512/1024 bits. Native width-one/six calls took 2.919 and 47.267 seconds and
reached remainder one. The complete process took 63.581 seconds with peak
resident memory 992,836 KiB. Its evidence is
`output/diagnostics/point-first-laurent-relative-point2/` and the adjacent
`point-first-laurent-relative-point2-process/report.json`.

All three exact prescribed points therefore have complete independent native
original-expression vectors. This does not establish an identity throughout
the integration domain, integration accuracy or performance parity. The
next compact-expression experiment must compare every coefficient, with the
same rational coordinates and no symmetry multiplicity. The original two
absolute-depth timeouts remain part of the cost evidence.
