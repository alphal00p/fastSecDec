# Phase-one remaining gates — independent source audit, 2026-10-05

This review compares the governing sections of `FIRST_PHASE_PLAN.md` with
`docs/REGRESSION_MATRIX.md`, current public source and tests. It does not turn
historical plan progress entries into current requirements or claim a new test
execution. The accepted native-alias implementation and its full workspace gate
are reviewed separately in
[the production audit](native-template-alias-production-independent.md).

The matrix contains **182 rows: 100 Covered, 81 Retired and one Partial**.
The remaining Partial row is `test_integrals.py:6113`, the difficult double-box
case. Its full vector now has an independent reference; uncertainty calibration
remains open. Retired rows mostly concern removed backend switches, Python
execution, projector caches, particular sector IDs and old text/serialization
formats. Their scientific replacements are identified in the matrix. I found
no additional absent numerator, subtraction, integration, reference-result or
checkpoint capability concealed by those retirements.

## Scientific acceptance still open

| Gate | Current evidence | Required next evidence |
| --- | --- | --- |
| On-shell triple box | All native inputs load. The original-card 1,800-second trial timed out at representative 80 without an artifact. The earlier source-bound production representative passes every order at three independent points. The later interleaved named candidate completes representative 80 and native IR construction, but its original-expression oracle and cold comparisons remain open. | Complete original-card generation and portable reload, all requested Laurent coefficients, then full-support numerical evaluation and independent integral-level evidence. Neither a successful representative nor evaluator construction closes this gate. |
| Off-shell scalar and rank-two triple boxes | Original and exact native projected families generate and integrate complete vectors. Both now have independently audited complete references, exact provider-bit transport, and separate original/projected native comparisons. The [rank-two audit](projected-rank-two-reference-independent.md) retains numerator/Gamma normalization and maximum absolute pulls below 1.727. | Prespecified convergence/calibration for both. Scalar and rank-two finite-part reference uncertainties are about 2.5% and 1.67%, respectively, above the one-per-mille target. |
| Hard four-loop full orthant | Native full generation and all 699 kernels complete with `[-2,-1,0]` covariance. The [independent reference audit](hard-four-loop-reference-phase-independent.md) now accepts all four provider orders `[-3,-2,-1,0]`; an independent native generation certificate proves zero through −3 with complete map coverage. | Convergence/calibration and the finite-part target: reference relative SE is about 0.569%. The old numerical estimate is unchanged; its full-union comparison retains MissingEstimate at −3 and is globally ineligible. The separate zero proof is not padding. |
| Double box and Issue 1 | Both now have independent complete-vector references. Issue 1's ordinary `together=True` fixture addresses the earlier disteval cross-sector variance omission; the older fixture remains Unverified. | Difficult-case repeated-seed/error calibration and highest-requested-order accuracy. A Checked reference establishes its audited provider route, not universal calibration of either estimator. |
| Six massive families and numerator controls | Six independent references and the 72-row holdout pass; one-loop masters/reduction and coupled-sunset analytic controls pass. | Prespecified higher-work/shift controls and final convergence/performance acceptance. No repeat of already passed small physics proofs is needed merely to fill a matrix cell. |

The older [campaign schedule](remaining-scientific-campaigns.md) deliberately
retains its original evidence chronology. Its double-box/Issue-1 table entries
predate the newer checked references; use the subsequent reference reviews and
fixtures for their current status. Subsequent scalar, rank-two and hard-reference
closures likewise supersede missing-reference entries in that historical
schedule. The hard reference preserves the full orthant and U factor, not the
historical single-sector diagnostic. The exact graph/direct-UF double-box identity
already closes input equivalence; a separate parser path does not create a new
physical integral requiring a duplicate numerical campaign.

## Concrete implementation and scope items outside the matrix

1. **Geometry caching and caller-owned parallel scheduling are implemented.**
   The plan explicitly calls for geometry caching by domain/canonical support
   and parallel independent charts/cones. The original serial decomposition
   API remains unchanged. `generation/support.rs` caches native source
   polynomial supports within a generation; pulling triangulation retains its
   native local face cache. The independently reviewed
   [cache core](geometry-cache-independent.md) adds bounded caller-owned reuse
   of complete decompositions, with all 28 sector tests passing. Generation
   entry-point adoption subsequently passes its independent audit and the
   combined 305-test workspace gate through `GenerationContext`. Parallel
   chart/cone work subsequently passes its 38-test sector gate, independent
   ten-test rerun and source audit, using the same native mathematics and
   private caller-scheduled jobs. The subsequent
   [cache/context dispatch adoption](parallel-generation-dispatch-independent.md)
   passes 41 sector tests and eight context tests, including complete analytic
   coefficient/map equivalence. The
   [CLI adapter](cli-geometry-dispatch-independent.md) passes 49 CLI tests and
   real terminal cancellation/resize/cleanup checks. It uses the application's
   existing Rayon pool primitives; the library owns native plans, opaque
   completions and deterministic admission, never an executor. Cache hits skip
   dispatch while reassessing each integral's domain and mapping. Free/default
   library generation and the one-worker CLI route remain serial. These close
   the implementation/interface item; measured geometry speed and retained
   memory remain separate performance work. Symbolic representatives are not
   made parallel by this chart/cone adapter.

2. **General no-threshold input is broader than the current endpoint admission.**
   `generation/domain.rs` safely rejects upper-cube zeros and unresolved
   mixed-sign boundary geometry. The explicit
   `upper_cube_endpoint_requires_affine_charts_even_with_assertion` test rejects
   `(1-x)^(-1+eps)` on the unit interval, despite its lack of an interior
   threshold. General affine endpoint charts are not implemented. No shipped
   example was identified as requiring this additional chart type, and the
   plan intentionally defers general domain splitting/contour/GCAD work.
   Nevertheless, final phase-one scope must retain this limitation explicitly
   or supply a reviewed endpoint-chart extension; do not equate the
   `assume_no_threshold` assertion with resolution of endpoint geometry.

3. **The additive sector content identity API is now implemented.** The plan says
   content-based sector identities. Current generated kernels and
   `KernelResultManifest::from_kernels` use original zero-based slice IDs,
   qualified by the complete kernel content hash. Public integration callers
   may provide their own stable IDs. Selection/resume are safe and tested with
   those current identities. A new standalone per-sector content ID supports
   reuse across different parent artifacts. It is an additive derived
   accessor/hash over native IR, ordered layout, numerical policy and retained
   semantics, preserving all current indices and checkpoint meanings. The
   [implementation and independent review](sector-content-identity.md) pass
   four focused tests, formatting and library Clippy. The digest does not claim
   CAS equivalence or change existing artifact bytes; machine-code caching
   would need additional architecture/backend identity.

4. **The identified interactive presentation gaps are now addressed.** Plain/JSON status,
   numerical failures, real SIGINT, partial resume and scoped results have
   process tests. Dashboard setup/error/drop has terminal-restoration code,
   and subsequent [actual PTY controls](terminal-policy-results.md) now verify
   resize, cleanup, monochrome output and key cancellation. A shared policy
   handles `NO_COLOR`, plain runtime errors and boundary/dashboard colors.
   Compact public-status rendering fixes the observed crowded small window.
   Argument parsing/help remains owned by Clap. This Linux PTY evidence does
   not establish every terminal emulator or platform; final platform
   qualification remains separate.

No production change is proposed by this audit. In particular, automatic family
projection in the CLI is an optional optimization: native prepared-family APIs
exist and preserve the original route/fallback. It is not required to make
original-graph scientific equivalence true. Future Python bindings, contour
deformation, GCAD, arbitrary complex masses and CBC construction remain outside
the agreed first phase. Native general map/domain/branch records are already
retained and inspected; their availability does not implement those algorithms.

## Performance and final verification

The eight-physical-core triangle/box native-only seven-seed campaign and the
individual-sample latency diagnostic are completed evidence. The paired
eight-core reference remains unavailable after its instance-limit failures.
The small matched one-worker comparisons retain backend, timer and persistence
differences and do not satisfy the entire matched 5% acceptance criterion.
Required representative generation/load/fixed-work/time-to-accuracy, difficult
sample-tail costs, seven/three paired repetitions and platform gates remain.
The target is always the largest signed requested epsilon order; no substitution
of a lower pole or extrapolation from fewer workers is valid.

After the original on-shell capability gate, prioritize prespecified difficult
case convergence/calibration before broad tuning; the reference vectors listed
above are now present. Retain the explicit endpoint-scope limitation, finish matched performance and
executed/qualified platform evidence, and perform the final native-reuse,
dependency and CLI audit. Existing controlled failures and source/build
identities must remain part of that record.

## Full-graph wrapper review

The reviewed `output/probes/production_alias_fullgraph.sh` preserves the original
`examples/runs/triple_box.toml` (`s=t=-1`, null external legs, `D=4-2*eps`, unit
measure, requested order zero). It bounds generation by 1800 seconds, inspection
by 180 seconds and complete integration by 180 seconds, each under a 30 GiB
address-space cap. Integration explicitly selects FullIntegral, 1024 points,
eight shifts, seed 20261004, Kuo38005 and two workers. Failed stages stop later
work and all successful native vectors/designs remain in saved results.

The author added absence checks for every stage directory and artifact,
checkpoint and result path after review. The launch manifest must bind the
wrapper, process timer, original card/DOT/model/parameter card and frozen release
binary/build identity, with pre/post checks. Actual method, periodization,
allocation and complete covariance are verified from native saved output;
invocation flags alone are not completion evidence. The subsequent frozen trial
timed out after 1,805.689 seconds including termination grace; all 26 immutable
postchecks passed and no artifact, inspection or integration result was produced.
See [the retained attribution record](native-series-fullgraph-attribution.md).
No whole-graph success is asserted here. The later test-only representative
candidate and its still-open oracle gates are tracked in
[the interleaved-face audit](native-interleaved-face-independent.md).
