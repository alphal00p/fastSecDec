# Native named coefficients: public opt-in integration

The public `NativeNamed` route now starts from admitted mapped terms before
physical endpoint subtraction constructs a density Atom. `Physical` remains the
default. This slice preserves native Series arithmetic, native differentiation,
native aliases and the existing evaluator/artifact/precision owners. The public
small-case gates and the subsequent 370-test combined workspace gate passed.
The [public reconstruction of the captured on-shell representative](native-named-public-actual-independent.md)
also passes complete independent coefficient and cold-artifact comparisons.
The original full graph remains a separate acceptance step.

## Module and API boundary

- `generation/coefficient_first.rs` owns the native relative-width controller,
  checked caller caps, exact unregulated-endpoint fallback and cancellation.
- `coefficient_first/compose.rs` preserves the tested Taylor/IBP operation order,
  equal-prefactor grouping and native scalar-Series products. Unknown-zero
  branches retain their native remainder; a successful result requires the
  actual native absolute order to exceed the signed requested maximum.
- `coefficient_first/requests.rs` and `requests/interleaved.rs` own per-attempt
  native source/request caches and flat native alias lowering. Restricted
  interleaving uses admitted regular faces; other argument shapes retain native
  derivative-then-substitution. No custom symbol hooks or global body registry
  are installed. The native empty-attribute plus `is_exportable` guard rejects
  foreign metadata after declared input handles have been skipped.
- `generation/coefficients.rs` selects the route and reports work through the
  existing `GenerationEvent` owner. Its result rejoins common assembly before
  root-only multiplicity, dependence classification, exact offsets, order
  padding and chart/kernel association.
- `generation/conditioning.rs` owns checked coordinate profiles. Physical
  results retain actual remainder rows. Named results derive a componentwise
  maximum degree row from every original mapped endpoint power; the exact
  physical fallback retains its actual rows.

`GenerationOptions::coefficient_expansion` contains the method and optional
maximum native attempts, relative width and distinct derivative/face requests.
`None` adds no cap. These extra limits apply only to named composition, while
ordinary subtraction degree/piece limits also remain in force. An exact
unregulated-endpoint fallback uses the existing physical limits and admission;
resource exhaustion, cancellation or failed Series does not trigger a fallback.

The additive coefficient progress event carries a typed stage and current
attempt counts. They reset on retry. Width/attempt zero means work has not
started, not Laurent coverage. Formal piece counts are not physical terms;
successful physical fallback reports zero formal pieces. Fallback is explicitly
observable before completion. Named work, including fallback, contributes only
to the exclusive `CoefficientExpansion` phase. Physical events and their timing
boundary through multiplicity remain unchanged.

The fresh `GeneratedSector::conditioning_basis()` distinguishes retained
remainders from a mapped endpoint bound. Both schedule the existing precision
heuristic; neither certifies floating-point error. On the open unit cube the
componentwise bound dominates physical remainder rows. Artifact v3 continues to
persist the same numerical rows/degree and native exact evaluator bytes, without
adding a descriptive basis field or new zero/real facts. Loaded artifacts do
not establish the descriptive basis of an old profile. Conservative cold-load
precision behavior is unchanged.

## Focused evidence

The private extraction first passed 14 new tests plus 15 physical subtraction
controls, with two existing ignored diagnostics. Both no-run builds succeeded;
the second added an explicit nonempty guard to nonzero vector controls. Private
formatting and main library/test Clippy passed. The immutable eight-file source
archive and binary/build evidence are at
`output/diagnostics/coefficient-first-private-source-1/`.

The public matrix then passed 40 tests, followed by an eight-test public target
rerun adding cancellation to zero after chart admission: **41 distinct focused
tests, zero failures**. Evidence is retained at
`output/diagnostics/coefficient-first-public-focused-1/` and
`output/coefficient-first-public-zero-followup.log`.

| Public gate | Evidence exercised |
| --- | --- |
| New generation target, 8 tests | Complete signed vectors for Taylor/IBP and negative/positive maxima; physical comparison; conservative rows; cold/warm/dispatched contexts; changed bodies across calls; multiplicity; exact and zero-only chart support; progress retries/cancellation; caller limits; failing and successful physical fallback |
| Native alias target, 2 tests | Both methods; hidden complex bodies; native 512-bit independent original-density Series reference; negative/positive maxima; fresh and decoded v3 kernels; cloned workers; default and forced precision replay; interior and `1e-80` points |
| Gamma regulator target, 1 test | Both methods; complete analytic Gamma/endpoint vector through positive order; literal underscored symbols |
| Artifact process target, 2 tests | Both methods; ordinary public generation/compile/save; separately executed cold reader; complex Gamma/log dependence and weighted MPFR replay |
| Kernel artifacts, 6 tests | Native IR and codec validation, legacy v1/v2 bytes/identity, hidden complex rejection and immutable program transport |
| Existing generation context, 8 tests | Physical default and caller-owned geometry behavior remain covered |
| Private native modules, 14 tests | Full-vector identity, unknown remainder/zero coverage, exact fallback, local request/cache/namespace ownership, checked caps and first-error cancellation |

The first public compile was retained: two test API errors confused a native
geometry completion with a `Result` and used a trait name as a zero predicate.
They were corrected without changing production arithmetic. The first public
Clippy run found two collapsible test `if` statements; equivalent let chains
closed them. Final formatting and main library/test Clippy passed (10.75 s).
The final 16-file source snapshot is
`output/diagnostics/coefficient-first-public-source-1/`; its record notes that
the final style-only test delta is to run in the combined workspace gate.

No time above is a performance comparison. This is public small-case evidence,
not full on-shell graph completion, a default switch, or benchmark parity. The
accepted captured-representative candidate/independent-oracle evidence remains
separate; the subsequent public reconstruction now passes complete-vector and
ordinary cold-artifact controls in the linked independent audit.
