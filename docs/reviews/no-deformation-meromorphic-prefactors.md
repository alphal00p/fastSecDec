# Meromorphic prefactors in the rational endpoint bridge

The certified one-dimensional bridge now admits exact Q(i)(epsilon) factors
multiplied by integer powers of native Gamma(a+b*epsilon), with rational a,b.
The complete original prefactor remains in symbolic subtraction and Laurent
expansion. A selected regular epsilon value is proof evidence only; it never
replaces the prefactor in the density or removes its poles from the result.

All chart endpoint inequalities first establish one open convergence strip.
Native polynomial root-count bounds and exact Gamma pole-family counts then
give a finite rational candidate set containing a common nonpole point for
every term. Exact denominator values and Gamma pole distances certify a
nonempty complex neighborhood inside the strip. Reciprocal Gamma zeros are
regular and do not require evaluating a singular Gamma function at the witness.
This is a local meromorphy certificate, not a claim that the whole strip has no
poles. Unclassified functions, noninteger Gamma powers, nonlinear arguments and
constant Gamma poles remain explicit unsupported cases. Auxiliary-regulator
restriction and the general algebraic atlas are outside this milestone.

Symbolica owns rational/algebraic-field conversion, recombination, polynomial
arithmetic, derivatives and Laurent expansion. The initial coefficient field
is checked before and after conversion: generic numeric Atoms are not silently
accepted as complex rationals. In the trivial Q extension, native polynomial
conversion avoids the recorded preferred-zero-generator conversion limitation;
the native algebraic context is retained for Q(i) and its selected embedding.
Native Rational::floor truncates toward zero; the Gamma distance calculation
uses nonnegative absolute values before applying it.

An executable owner probe showed that excessive native polynomial exponents
can panic during conversion. Checked syntax bounds therefore precede conversion
and recombination, and also protect the existing numerator/unit polynomial
conversion. They bound nodes, depth, degrees, powers and possible expansion
size without manipulating coefficients or implementing polynomial algebra.
Caller-enlarged prefactor degree limits stay below half the native exponent
range. Conservative resource refusal is permitted even when a later symbolic
cancellation might have made an expression smaller.

Bound native numerator definitions are now exposed crate-privately from the
same registration loop that constructs FunctionMap. Record-local staging can
persist their native Atoms without exporting process-local Symbol IDs.
Coordinate-independent Laurent coefficients can materialize owned numerator
and derivative calls at fixed faces using these same native bodies. Malformed
tags/orders and remaining coordinates, regulators or physical parameters are
rejected. Bulk stochastic expressions remain composed.

The independent resolution reviewer and root checked the shared-witness
argument, Gamma distances, exponent limits, retained phases and native reuse.
Six new registered controls cover rational identities, shared witnesses,
Gamma/resource admission, exact helper reconstruction and a native HEPKit
graph comparison. Registered Cargo validation is recorded below.

The graph-derived B0(16;3,3), with mu-squared=1 and native Gamma/rGamma measure,
has three verified cells and six half charts. Its original quadratic F remains
one causal factor. The saved complete Laurent evaluator gives
pole `1+0i` and finite `0.3520815669978355+1.5707963267948966i`, agreeing with
native OneLOop to `5.55e-17`. Orders are the actual -1 and 0, with no manual
coefficient shift. The caller-owned Kuo/Korobov3 allocation used 16,384 points
and 16 shifts, seed 202610106102, and retained the full complex covariance.
The predeclared per-component check was six native standard errors plus
2e-10. This particular near-roundoff control is not a general variance claim.

The guarded reference process took 2.829 seconds with sampled peak RSS
463.7 MB. It exercised the native bridge/evaluator, not completed v15 artifact
publication or serial generation. The accepted final resource/definition
changes were compared with the immutable sampled snapshot by exact expression
and coefficient identities and restored evaluator values at three points.
Raw provenance and source digests remain untracked in
`target/no-deformation-prefactors/handoff.json` and `bubble-qmc-run/`.

Registered validation passes all 90 threshold tests, including the six added
prefactor controls, plus strict native library/test Clippy and workspace
formatting. The reference loop's iteration style and production use of the
definition getters were adjusted during registration; no lint exemption or
public-visibility widening was needed. The common gate log is retained in
ignored `target/no-deformation-prefactor-iteration-gates.log`.
