# Retained generation explainability

Each new native source chart retains a versioned, optional record at the return
from `mapping::map_terms`, before symmetry registration, multiplicity, endpoint
subtraction or Laurent expansion. It holds each nonzero combined mapped term's
coordinate-independent prefactor and ordered endpoint powers. The existing
native endpoint admission supplies exact rational `b` and `c` in
`t^(b + c epsilon)`, and the required Taylor coefficient count. This count is
not a surviving pole count: boundary coefficients may vanish. The original
source-chart coordinates and representative permutation remain authoritative.

The record keeps factored Symbolica Atoms; it does not expand a coefficient or
reconstruct a density from strings. To avoid duplicating potentially large
regular factors, it retains only their native Atom storage size. The prefactor
and powers therefore do not constitute a restorable complete mapped density.
Coordinate substitutions, exact geometry and the positive measure Jacobian
continue to use the existing retained owners.

The portable chart codec adds an optional field and omits it entirely when
absent. Existing artifacts keep their original bytes and identities; absence is
explicit rather than inferred from expanded coefficients. Loading new records
validates the version, regulator separation, coordinate dimension and prefactor,
then reuses native endpoint admission. It adds no polynomial manipulation or
independent rational-power recognizer.

Evaluator size is measured from the actual complete-vector program built in
`kernel::program`, including its shared alias map. Symbolica's public
`ExpressionEvaluator::count_operations()` reports the exact program's arithmetic
operations; the existing native encoder supplies its byte size. These are
pre-SymJIT counts, not scalar machine operations after real/complex lowering or
optimization. `JITCompiledEvaluator::as_bytes()` supplies the actual compressed
serialized SymJIT application size. It is not machine-code size and is absent
for the portable interpreter. Statistics are captured on compilation or cold
load, not reconstructed from coefficient counts. Complex evaluator outputs are
counted before their numerical real/imaginary split. No per-coefficient compiled
cost is assigned to shared expressions.

Python wrappers retain native `Arc<GeneratedIntegral>` ownership and indices for
chart, term and power views. Exact rational values become native Symbolica
Expressions with `Atom::num`; statistics copy only their small immutable native
record. Notebook inspection requires an explicit action and selected term index.
Bounded native LaTeX presents maps, Jacobians and prefactors; source names remain
available on demand. The all-sector overview reads recorded operation/byte
counts without constructing or evaluating kernels. Human CLI inspection gives
term counts and endpoint requirements, with formulas under `--expressions`.
Its existing JSON mode preserves full semantic transport and includes evaluator
statistics.

Independent source review accepts both the native capture/codec/statistics and
thin Python transport. Focused controls exercise a regulated `x^(-2+epsilon)`
term requiring two Taylor coefficients, source-chart symmetry without duplicated
multiplicity, cold round trips, explicit legacy absence and unchanged artifact
identities, malformed record rejection, and real/complex evaluator statistics.
Presentation controls verify selected-only term formatting, exact requirement
display, legacy absence and portable size absence without session access.
The final native metadata/artifact gate passed 17 controls. The portable-host
gate passed 41 controls: 12 artifact/metadata, 8 Havana, 5 discrete Havana and
16 QMC controls, with no failures or skips. Selected-view/lifecycle/report
controls pass 21 cases and marimo validation is clean. The rebuilt native wheel
passes all 81 maintained controls. A 24.985-second actual triangle UI session verifies rendered native map math, selected retained
metadata/size facts without sampling, QMC Cancel/Resume, same-kernel New
integration and Havana pilot/frozen-production lifecycles, with exact accepted
checkpoint prefixes and full vectors/covariances. All owned processes were
reaped. Four focused controls and separately labelled retained-data rendering
cover the subsequent CSS-only table spacing correction. Earlier portable browser
evidence remains bound to the pre-extension Wasm wheel and does not validate
these new APIs. Native UI evidence is retained under
`output/diagnostics/hepkit-rich-metadata-ui-1/run-1/`, with the later display-only
review under `output/diagnostics/hepkit-rich-metadata-presentation-1/`.

The new fields use existing Symbolica/Numerica algebra, evaluator and formatting
APIs. No dependency patch, alternate CAS, graph type, numerical estimator,
library-owned worker or independent one-loop reference is introduced. This is
inspection and storage work; it makes no new amplitude or performance claim.

A fresh native fixed-point gg→HH generation and cold reload completed in 58.461
seconds. The saved objects retain 30 charts, 54 mapped terms and 324 coordinate
powers; every native Taylor requirement is zero. The powers comprise 270 zeros,
24 epsilon, 24 one-plus-epsilon and six minus-epsilon entries. Root's independent
pure-data comparison confirms that all original kernel payload fields are
unchanged except the added metadata, and removing the new optional records
restores the original metadata exactly. Exact program byte sizes also agree.
The new content identity correctly includes the new retained facts. Evidence is
under `output/diagnostics/gghh-native-metadata-generation-1/`; this generation and
storage check performs no new integration or amplitude comparison.
