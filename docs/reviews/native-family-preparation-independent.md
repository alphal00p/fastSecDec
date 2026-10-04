# Independent native family-preparation review

This is the HEPKit/API review of the first library implementation, separate from
the author's design and test record in `native-family-preparation.md`. The source
review and subsequent 24-test focused gate are recorded below. It does
not claim that the CLI has enabled projection by default or that a portable
artifact already records preparation metadata.

The preparer borrows the original `feynkit_graph::IntegralFamily` through `Cow`
or owns the native `sector` result. Native `is_independent`, `partial_fraction`
and `sector` own rank, elimination and family reconstruction. Inspection of the
pinned FeynKit `integrals.rs` shows that `sector` keeps the original ordered loop
and external momenta and the same kinematics, and selects positive powers in
original denominator order. No routing transformation, new graph type, local
partial-fraction algorithm or duplicate estimator is introduced.

Admission is deliberately conservative: one exact coefficient-one term, no
negative powers, a smaller nonempty active support, unchanged momentum bases,
and native Atom equality of the original and candidate inverse-propagator
products. Failure of that last canonical check declines optimization; it does
not invoke a new simplifier or expand the complete density. Native state limits
and checked-power overflow fall back to the untouched original family; other
native errors propagate. A positive-power zero denominator is explicitly invalid
before either policy's early return. The state bound is not presented as a
wall-time or cancellation bound on a single native call.

`from_family_prepared` first reuses the existing full original label, numerator,
regulator and dimension admission. In particular, labels which projection would
discard are still checked for numerator/dimension/denominator collisions. Native
original U/F construction owns the private family-label checks. Those U/F atoms
are reused for a no-op; successful projection builds active U/F using the
original labels indexed by the native active support. This additional original
U/F work is explicit and should be measured before default CLI adoption. The
unchanged `from_family` and `from_graph` entry points retain their original-family
meaning.

The graph wrapper obtains its contracted scalar numerator once and multiplies
the existing measure multiplier once, then delegates. The preparer neither adds
weights nor reconstructs the raised-power measure. Its report retains original
powers, ordered active original indices and positive active powers. It describes
the complete physical integral, rather than a numerical selection of sectors.
Caller-owned generation, workers, checkpoints and runtime are unaffected.

The six drafted tests cover the repeated massive Gaussian moment against an
independent native-polynomial simplex integral, noncontiguous surviving labels
and a loop numerator, native multi-term/state-budget/no-op fallbacks, discarded
label rejection, checked integer bounds, and a native graph weight of 210 with
raised powers giving the expected value 70. The review requested one additional
invalid-zero-denominator assertion in the existing admission test. The report's
serde round trip checks this library record only; later CLI/portable integration
must still bind policy and original graph metadata without creating a second
map schema or changing the meaning of full-integral scope.

## Executed focused gate

The six preparation controls passed, including the added positive-power zero
input under both policies and a nonunit-coefficient fallback. The existing four
native-family entry controls and fourteen native-input controls also passed:
**24 tests total**. Logs are `output/family-preparation-focused-tests.log`,
`output/family-parametric-regression-tests.log`, and
`output/family-native-input-regression-tests.log`. The independent review checked
those named outcomes. The preparer's entry methods were moved into the small
`preparation/entry.rs` module without changing their contracts. No source-review
finding remains open for this library slice; default CLI adoption, preparation
metadata persistence and ordinary-family no-op performance remain separate gates.
