# Independent native family projection probe review

This is a read-only review of the ignored `projected_triple_box.rs`,
`triple_box_input.rs` and `native_qmc_diagnostic.rs` orchestration, following the
validated [public family entry](native-family-entry-independent.md). The reviewer
did not author these probes. Numerical execution is a separate coordinated step.

The input helper reuses native model/DOT parsing, kinematics and scalar binding.
It explicitly separates the shipped off-shell point (external virtualities −1,
s=t=−2) from the on-shell point (zero virtualities, s=t=−1), and verifies the
dependent fourth external virtuality with native scalar products. The selected
rank-two graph supplies its native numerator fragments.

Native `IntegralFamily::partial_fraction` and `sector` identify the independently
observed one-term, eight-active-denominator family with powers
`[1,1,2,2,1,1,1,1]`. The probe verifies the original and projected inverse
denominator products by native Atom equality and retains the same loop/external
bases. Unexpected coefficients, term counts or negative powers stop this fixed
experiment. The already weighted native scalar numerator, measure multiplier
and native fraction coefficient enter `ParametricIntegrand::from_family` exactly
once. No custom reduction, loop shift or polynomial identity engine is added.

Generation, domain checks, maps, subtraction, complete Laurent vectors and O2
compilation use the ordinary native pipeline. Stage callbacks preserve timings
and enforce the cooperative generation bound; a separate process watchdog is
still needed for a single long native call. The two representations need not
have matching parameter names, chart counts or kernel identities. Their numerical
comparison must preserve the complete physical coefficient vector and covariance
rather than require matching sector IDs.

The integration helper delegates its design, point generation, periodization,
weighted precision/replay, common-shift estimates and checkpoint to existing
native APIs. Caller threads own their evaluator/worker contexts. Accepted replay
maxima merge only after successful submission; a failed package's prefix is not
saved as accepted work. Soft timeout returns accepted coverage and its native
checkpoint. The projected command now emits this retained report before
returning a nonzero status for an incomplete allocation or unavailable
authoritative total.

No blocker was found for the fixed published-rule, full-integral experiment.
The reported preparation, numerical loop and observation times remain diagnostic
until execution conditions and both representations' complete numerical evidence
are recorded. This review does not authorize automatic graph reduction or infer
performance/scientific parity from the exact denominator identity alone.
