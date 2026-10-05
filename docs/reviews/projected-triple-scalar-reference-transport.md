# Off-shell scalar triple-box reference transport

The independently reviewed complete ordinary pySecDec observation is transported
as `examples/references/triple_box_offshell_scalar.json`. The original scalar
and native projected scalar computations are compared with it separately. All
four native means, standard errors and 16 covariance entries in each retained
observation remain unchanged; no estimates are pooled or reconstructed.

The external [numeric-only continuation](projected-triple-reference-numeric-continuation.md)
returned physical orders −3 through 0. Its ordinary sector-sum path samples all
1182 sectors before estimating each coefficient, with successive draws from the
same native RNG. The generated raised-power measure and `Gamma(4+3*eps)` occur
once in physical tuple member 2. The [independent outcome audit](projected-triple-reference-independent.md)
accepted scientific input, normalization, uncertainty path, complete allocation
and raw transport. It retains unknown cross-order covariance and does not
certify error calibration or convergence.

The ignored Rust writer `output/probes/record_projected_triple_scalar.rs` uses
the existing native `ReferenceResult`, versioned encoder/reader and `compare`
APIs. It checks the audited decimal values against exact provider IEEE bits,
all four order keys, positive statistical errors, power projection, original
source records, unchanged numerical settings and successful immutable-copy
postconditions. An initial audit JSON serialization rounded decimals to 15
digits; both reviewers caught this before writer execution. The final audit
retains raw round-trip decimals and exact binary64 bits. No rounded values
entered the fixture.

The writer first compares an Unverified candidate, then applies Checked only
after the independent audit and full-vector checks. Numerical rows are exactly
the same before and after that eligibility change. The native diagnostic pulls
are:

| Epsilon order | Original graph | Native projected family |
| --- | ---: | ---: |
| −3 | −0.952 | −1.723 |
| −2 | −0.773 | −1.301 |
| −1 | 2.815 | 2.022 |
| 0 | 1.131 | 1.207 |

The native observations share seed 20261004 and may be correlated with each
other. Each comparison uses the independently generated external seed 20261201
result; these are not independent original-versus-projected pulls. All physical
values and provider errors remain measured data, including the tiny nonzero
leading pole. Reported imaginary values/errors are zero and are preserved as
observations in provenance; only explicit real coefficient rows are transported.

The highest requested order is 0. Its external relative standard error is about
2.5%, above 0.1%; `highest_order_target_met=false`, `calibration_certified=false`
and `exactness=false` remain explicit. The 1,063,808 observed work count is scalar
summed-sector coefficient point evaluations, excluding setup and auxiliary work,
not native full-vector samples. No performance ratio follows from this continued
scientific run.

The pure-data writer passed using Rust 1.98.1 and the preserved optimized native
library. Evidence, unmodified original/projected estimates, comparisons before
and after Checked, and build/source hashes are under
`output/reference-fixtures/projected-triple-scalar-native-reference/`.
`output/projected-triple-scalar-reference-writer.log` records the comparisons.
The new fixture SHA256 is
`0129638e3a8913b2e9abc3542fb1ca852d00fe47dd4951c1e017e9d9e8763c0c`.

The existing CLI reference transport target passes **five tests, two ignored**
external recording probes. Its new regression checks all IEEE values/errors,
native measure/powers, full scope and order range, count units, measured
imaginary zeros versus unavailable covariance, source bindings, conservative
certification flags and exact native versioned roundtrip. The log is
`output/projected-triple-scalar-reference-tests.log`; the tests perform no
sampling or Symbolica work. The final independent writer/fixture review passed:
all eight provider mean/error bit patterns, encoded fixture bytes and build
hashes were checked; both complete native estimate objects including all 16
covariance entries match the retained originals exactly. All comparison rows
are unchanged by the Checked eligibility update. This slice is ready for the
coordinator's documentation/fixture milestone. Root's focused
`cargo clippy --locked -p fastsecdec-cli --test multiloop_references -- -D warnings`
also passes; `output/projected-triple-scalar-reference-clippy.log` retains the
existing upstream Symbolica warning separately. Formatting and diff checks
pass, without repeating the already accepted full scientific workspace gate.
