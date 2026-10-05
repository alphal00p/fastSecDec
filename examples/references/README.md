# Independent multiloop references

The six massive versioned `fastsecdec-reference` files retain the finite real
coefficient and positive reported standard error from independently generated
pySecDec C++ packages. They correspond to the equally named native run cards:
mass one, external virtualities minus one, `D=4-2*eps`, unit propagator powers and
normalized per-loop measure `d^Dk/(i*pi^(D/2))`.

`double_box.json` additionally preserves the full real vector from orders minus
four through zero for the massless on-shell card at `s12=s23=-1`, with the same
normalized measure. It comes from an independently generated pySecDec C++ package
and retains every reported standard error, including the tiny measured leading
pole and its nonzero uncertainty. The constituent `-Gamma(3+2*eps)` factor is
already applied; it must not be multiplied a second time. The full native
64-shift comparison agrees within 1.27 combined standard errors at every order.
See the [attempt record](../../docs/reviews/remaining-reference-attempts.md) and
[independent audit](../../docs/reviews/remaining-reference-independent.md).
The older uncertainty-free target in `examples/targets/double_box.json` remains
historical evidence and is not overwritten by this reference.

`issue_1_together.json` retains all three orders `[0,1,2]` from ordinary
`IntegralLibrary` with `together=True`: sectors are summed before native QMC
computes uncertainty. The older `issue_1.json` remains Unverified because its
disteval route omitted cross-sector covariance. These observations are kept
separate. The checked fixture does not certify error calibration or the
epsilon-two one-per-mille target.

`triple_box_offshell_scalar.json` retains the complete `[-3,-2,-1,0]` vector
for the scalar off-shell triple box, with massless internal lines, four external
virtualities `-1` and `s12=s23=-2`. The native projection has coefficient `1` and gives
eight active propagators with powers `[1,1,2,2,1,1,1,1]`. The returned physical
vector already includes `Gamma(4+3*eps)` once. Original and projected native
vectors are compared separately, preserving their complete covariance; all
pulls are below 2.816 combined standard errors. They are not pooled. The finite
reference coefficient is `2.283413488400494 +/- 0.05718537938081278`, still above
one-per-mille relative uncertainty. Four logged `8311 × 32` allocations give
1,063,808 scalar summed-coefficient point evaluations, excluding auxiliary
work; this is not a complete-vector sample count. See the
[independent audit](../../docs/reviews/projected-triple-reference-independent.md).

`triple_box_offshell_rank2.json` retains the same four orders and off-shell
kinematics for the supplied rank-two numerator. Its physical tuple includes
`Gamma(3+3*eps)` once. The separate original/projected comparisons retain their
covariance and have maximum pulls below 1.727. The finite value is
`0.6263784670955248 +/- 0.01044026507101012`, about 1.67% relative uncertainty.

`four_loop_hard.json` covers the complete nine-dimensional positive orthant
with density `U * F^(eps-3)` and unit prefactor. The ordinary provider sums all
2,676 sectors before estimating each coefficient, retaining orders
`[-3,-2,-1,0]`. Its finite coefficient is
`-150.88953098736903 +/- 0.8585945824027943`, about 0.57% relative uncertainty.
The historical native vector and its complete covariance still have three
orders `[-2,-1,0]`: comparison reports `MissingEstimate` at minus three and
remains globally ineligible. A separate native symbolic generation proves zero
through minus three; it does not pad that numerical observation or change the
provider's reported zero standard error into an exact classification. Common
coefficient pulls are below 1.629. See the
[independent audit](../../docs/reviews/hard-four-loop-reference-phase-independent.md).

`Checked` records the independent source, input, normalization and transport
review. Comparison eligibility additionally requires complete matching
observations and explicit comparison evidence. The label does not certify
convergence, calibration or matched performance. External imaginary values and
errors are retained when supplied; unavailable observations stay null. Where
the older reports do not expose actual lattice sizes or
evaluation counts, those fields remain null.
Requested `maxeval` is not presented as measured work.

Each file preserves the command, source revisions, versions, raw-report digest,
native/external input digests and sampling limitations. See the
[numerical record](../../docs/reviews/massive-multiloop-diagnostics.md) and
[independent audit](../../docs/reviews/massive-multiloop-reference-independent.md).
Raw reports, generated packages and build output remain under ignored `output/`.

Native callers use `fastsecdec::reference::read_reference` and `compare` directly.
The CLI accepts a file through `[reference] path` in a run card or `--reference`;
normalization, kinematics and independent-sampling evidence remain explicit
comparison inputs. Selecting a new file through an override clears evidence
attached to a different target. Comparison never changes integration or stopping.

The ordinary `multiloop_references` test checks these transports, uncertainties,
projection and current native source identities without external software. Its
explicitly ignored recorder uses the existing raw reports and native reference
API to write candidate files under `output/reference-fixtures`; it does not run
pySecDec or overwrite these frozen files. The ignored `massive_multiloop` CLI
harness generates and integrates all six cards and reports the library's full
typed comparisons. Its five-standard-error threshold is an initial correctness
check, separate from convergence acceptance.
The additional double-box transport regression checks all five coefficient
orders, the measured zero-residue observation, exactly-once prefactor ownership,
explicit imaginary projection and native input hashes. The fixture does not
certify difficult-case convergence or uncertainty calibration.
