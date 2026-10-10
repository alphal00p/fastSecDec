# Local monomial resolution and resumable proof frontier

This extends the checked monomial blow-up producer with repeated local steps.
It resolves the marked cosupport of a supplied, verified monomial ideal in an
SNC boundary. It does not implement the general BM invariant, real integration
charts, algebraic branch normalization, or final endpoint certificates.

Every target generator is associated with its carried residual through an
exact native quotient-ring identity. Positional association would be wrong
after native ideal normalization removes or merges generators. The remaining
residual ideal must again be the unit ideal; no individual residual generator
is assumed invertible. Pulled units, absent strict divisors and an exceptional
equation that is invertible on the current open are absorbed into that residual.

For an inclusion-minimal monomial center C with sum(alpha_i)>=d, the new
exceptional exponent beta=sum_C(alpha_i)-d is strictly smaller than each
center exponent. A pivot chart replaces its pivot exponent by beta and leaves
other surviving powers unchanged. An identity open avoiding the center removes
at least one positive center exponent. Thus the sum of active exponents strictly
decreases in every child. The implementation checks that decrease with checked
integer arithmetic. This is the termination argument for the monomial stage
in BM2008 section 5 II.A, not a replacement for the general resolution invariant.

The caller chooses which pending proof node to advance. Stable ancestry paths
identify nodes independently of scheduling. Private states retain the actual
parent cover, chart and controlled-transform owners. Completion requires all
children of every expanded node and a native EmptyCosupport certificate at
every leaf. Resource exhaustion preserves the frontier and incomplete inner
evidence; a later call may supply a larger budget. An incomplete inner stage is
currently recomputed. This is in-memory resumability, not a durable checkpoint
or a claim of bounded process RSS.

Six focused native controls cover generated exponent families, every pivot,
unit-ideal combinations, quotient-equivalent residuals, nonlinear guarded
charts with empty exceptional divisors, overflow and resource refusal,
reordered advancement and stop/resume. The ignored direct build passed all six
in 0.50 seconds and strict Clippy. Root and the independent runtime reviewer
checked the mathematical rank, exact generator associations, completion
ownership and stated scope. Registered Cargo validation is recorded below.

Native reuse remains the registered Symbolica polynomial, ideal, relative
derivation and matrix operations; no additional algebra implementation or
worker pool was added. Provenance is retained in ignored
`target/no-deformation-monomial-iteration/handoff.json` and its source/log digests.

Registered validation passes all 44 resolver tests. The combined prefactor and
iteration gate passes all 90 threshold tests, strict native library/test
Clippy and workspace formatting. Logs are retained in ignored
`target/no-deformation-iteration-cargo-gate.log` and
`target/no-deformation-prefactor-iteration-gates.log`.
