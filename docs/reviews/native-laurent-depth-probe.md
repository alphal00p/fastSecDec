# Native Laurent-depth attribution probe

This test-only experiment follows the bounded on-shell triple-box timeout in
[the scientific campaign](onshell-triple-box-and-issue-one.md). It does not alter
the production expansion strategy or claim that the whole Laurent timer is
native series time: template preparation, cached coefficient retrieval and
coordinate-image restoration also contribute to that timer.

Native API and source checks:

- Symbolica `src/atom/core.rs` exposes `series` with `SeriesDepth::absolute` or
  `SeriesDepth::relative`; `src/poly/series.rs` exposes native trailing exponent,
  absolute order, relative order and coefficient iteration.
- `src/derivative.rs:376` starts absolute order zero at a relative depth of one
  and doubles the working depth until the native absolute bound is sufficient.
  The source performs another `series_impl` call for each retry. No public
  reusable cross-call series cache was found in the inspected API.
- The frozen Pathfinder `src/subtraction_formula.py:1982` and `:2314` call the
  native series API with the known coefficient count and
  `depth_is_absolute=False`. This motivates measuring explicit native depth
  selection; it does not justify assuming a leading pole before native
  cancellation has been evaluated.

The disconnected-in-production hooks live under `cfg(test)`. The capture uses
the actual graph admission, parameterization, geometry, mapping, complete-density
symmetry and subtraction code. It skips only earlier Laurent calls, captures
the selected native expression/template and all opaque coordinate images with
Symbolica's native Atom export, then returns `GenerationError::Cancelled`.
An additional active-capture barrier rejects a missing target or zero-chart input
before generation could return a result. A thread-local guard clears capture
state on scope exit and unwinding. No incomplete integral can be returned from
this mode.

The initial target is representative index 38, displayed ordinal 39; source
geometry index and multiplicity are recorded separately. The fixture probe
checks the original 2112-chart/1026-representative counts. Index 45/displayed 46
is a second candidate if the first does not explain the bounded failure.
These identifiers are local to this traversal.

Each replay imports the same native template in a fresh process. The baseline
calls native absolute order zero. The candidate requests native relative depth
one, reads its native lowest stored exponent and absolute bound, and requests a
second relative depth only if needed to cover the requested absolute order.
Native Symbolica owns every expansion, cancellation and coefficient operation.
The experiment explicitly checks the final absolute bound and rejects
fractional Laurent output. It introduces no series algebra implementation.

Preparation, series and coordinate-image restoration are measured separately.
Restoration repeats the production native replacement and bounded small-expression
rational cleanup; it does not expand polynomials. Native Atom files preserve
both template coefficients and restored coefficients. A separate comparison
requires the full order lists and every native canonical coefficient to agree.
Export time, final multiplicity, evaluator compilation and numerical integration
are outside the reported native series timing.

Ordinary controls cover high poles, cancellation before the leading term,
negative requested orders, a positive-only expression, exact zero, fractional
rejection, capture with a preceding skipped representative, missing-target
cancellation and normal generation after guard reset. All three ordinary tests
passed in 0.15 s. The current production behavior remains unchanged.

## Captured representative and first replay

The optimized diagnostic binary was built from FastSecDec
`98de4faaac606d6af1917cf620d5a03be40eada3` plus the retained test-only hook/source
archive. `output/diagnostics/laurent-capture/` stores that source archive and
hash, base commit, lock hash, binary hash, Symbolica revision/local source patch,
build log and ordinary control log. Source and native dependency evidence are
kept separate from the earlier `561657b` production trial.

Capture passed in 6.173 s with 187,616 KiB sampled peak RSS. Representative
index 38/displayed 39 maps to original geometry source index 46 and multiplicity
two, with nine integration coordinates. The original subtraction expression is
14,505,102 native Atom bytes; its opaque template is 5,736,028 bytes with 201
coordinate images. Template preparation itself took 0.349 s. The native export
includes the original expression, template, regulator and 201 image/symbol
pairs. Fixture checks confirm 2112 charts and 1026 representatives, the shipped
card/graph digests and the exact native input-density digest. Independent HEPKit
review checked the successful controls, fixture metadata and all 405 native
export files.

The fresh-process absolute-order-zero replay passed its 180 s bound. Native
series time was 148.974 s and coordinate restoration 4.445 s; whole process time
including import/export was 155.289 s, with 1,767,544 KiB sampled peak RSS. The
native absolute truncation bound is one and output orders are `[-5,-4,-3,-2,-1,0]`.
The corresponding template-coefficient Atom sizes are
`[1116,30539,353862,2389783,10897372,36898483]` bytes; restored sizes are
`[6553,199577,2410180,16504347,75575971,256030180]` bytes. Thus native series work
dominates this representative, but the resulting full expression size is also
a concern.

The relative strategy completed its native series in 37.332 s: relative depth
one took 4.581 s and reported trailing exponent `-5`, relative width one and
absolute bound `-4`; a second native request with relative depth six reached
absolute bound one. Restoration took 4.267 s. Whole-process time was 43.426 s,
with 1,281,632 KiB sampled peak RSS. All six template coefficients and all six
restored coefficients are exactly equal to the absolute baseline as native
Atoms; the separate import/comparison process passed in 27.308 s. The roughly
fourfold reduction in series time is a single-representative diagnostic, not
whole-generation performance parity. The 256 MB finite coefficient remains in
both representations.

The initial fresh import emitted a warning that `symbolica::gamma` had been
exported with callbacks but imported without them. Independent review traced a
possible lazy-initialization ambiguity: `State::import` snapshots the symbol
count before `SymbolBuilder::build` initializes global native symbols. A cold
control then imported the actual template before any symbolic construction and
used its existing Gamma symbol directly. `Gamma(5)=24` passed before explicitly
requesting Gamma initialization; its derivative at one and series for
`Gamma(1+eps)` through order four also matched the known/direct native results.
A separate explicitly initialized process produced byte-identical canonical
template and control expressions. Thus the warning is a confirmed false positive
for this capture, and the original paired results remain valid. Future replay
entry points explicitly initialize native special functions before import.

The cold/control source and logs are
`output/probes/laurent_import_control.rs` and
`output/diagnostics/laurent-capture/import-{cold,initialized}.log`; canonical
identity hashes are in `import-identity.sha256`. The template's matching SHA-256
is `64ae587040dc1d5753ff424a44cc53e9d0bcfc58371b0ba4c6ee43eb5c7a5f59`, and the
control-series hash is
`c8ccd9f8fa83967923fcf40be8ecdfdbeebc5f19cabb59596a6d0db15dbba97d`.
The added explicit-initialization source lines postdate the retained paired
binary; the raw binary hash and source archive identify exactly what was timed.

## Reference direct-path representation

The actual direct builder is `integrand.py:7827`,
`build_explicit_sector_formula`. For singular sectors it constructs endpoint
assembler expressions over formal regular coefficients, obtains those
coefficients, then substitutes all derivative/source slots into the final
outputs before compiling one explicit evaluator. This intermediate symbolic
organization does not require a dual U/F runtime evaluator.

The important preparation steps are `integrand.py:6762`,
`_g_coefficients_by_symbolic_diff`, and `:6968`,
`_two_stage_derivative_fused_components`. They group requested Taylor/epsilon
coefficients by boundary and zero-coordinate subsets, compute needed derivatives
of regular residual polynomials, and form regular epsilon coefficients before
local-coordinate differentiation. The regular epsilon coefficient is expressed
using fixed base powers and powers of a combined logarithm, divided by the
factorial; a final coefficient substitution then feeds the endpoint assembler.
The reference therefore avoids a general Laurent expansion of the entire
already differentiated, Taylor-subtracted coordinate expression. Earlier
`_regular_function_coefficients`/`_instantiate_regular_template` helpers in
`subtraction_formula.py:2025`/`:2063` show the related cached small-template
approach, but they must not be confused with the active direct builder.

The current Rust route applies native derivatives/subtraction first and then
constructs the opaque epsilon template. This explains the representation
difference to investigate if depth selection alone is insufficient. Any proposed
regular-coefficient composition must continue to use native Symbolica series,
derivatives and coefficient operations, rather than copying the reference's
Python sparse-series dictionaries or assuming a pole depth. The next scientific
comparison is the existing native projected-family on-shell input, to determine
whether its seven-dimensional raised-power representation avoids this bottleneck
before broader production orchestration is considered.
