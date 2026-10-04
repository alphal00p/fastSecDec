# Regression coverage audit — 2026-10-04

This independent, read-only audit reviewed all 79 rows marked Pending or Partial
in `docs/REGRESSION_MATRIX.md` at the start of the review. The reference is
FastSecDecPathFinder `582d8c7f6dde9bf750750d4c2a2d85a94ce940cd`. Evidence comes
from the original test bodies, current Rust implementation and test bodies, and
the recorded successful milestone gates. No scientific runtime was rerun for
this inventory, and the matrix itself was not edited.

The accepted foundation gate was `d012d50` (176 tests, formatting and Clippy).
The subsequent `b94053f` milestone passed the nine reference-library tests and
22 focused CLI tests, including two fresh-process reference tests after the
native-TOML fingerprint and display changes, plus formatting and Clippy. The
completed double-box run used 64 shifts and 6.68 million evaluations without
evaluation failures; its leading coefficient was -0.000181 +/- 0.000589,
consistent with the independently established exact zero. Its higher
coefficients remain uncertified.
All recommendations below are about demonstrated behavior, not performance
parity or proof that an arbitrary integral's error estimate is reliable.

The proposed disposition of these 79 rows is 32 Covered, 31 Retired, 11 Partial
and 5 Pending. Retirement is narrowly justified below and does not remove any
of the remaining scientific acceptance gates.

## Findings that should drive the next scientific work

1. **Complete the multiloop numerical coverage.** The six smaller massive
   two-/three-point graphs currently have native loading/routing checks, not
   complete native generation and numerical comparisons. The original generation
   test covered `kite_2loop`, `self_energy_3loop`, `three_point_2loop`,
   `three_point_3loop`, `three_point_2loop_6line`, and
   `three_point_3loop_8line`; the optional numerical test covered the first two.
   Native sector counts need not match the old decomposition. Require valid
   retained maps, complete Laurent vectors, and independently sourced normalized
   coefficients instead. `issue_1` and the hard four-loop direct fixture also
   remain scientific/end-to-end performance gates, regardless of their successful
   input and exact geometry tests. Keep full-U/F geometry stress timing separate
   from legitimate F-only generation of a regular U power.
2. **Add a coupled two-loop numerator reference.** The native product-of-two-loop
   Lorentz-moments test is useful but does not cover the reference's coupled
   propagator `(k1+k2+p1)^2` and numerator `k1·k2+2 k1·p1`. A pointwise
   parametric-density comparison at several exact parameter/regulator values is
   a useful first gate; the shipped double-box numerator still needs a complete
   numerical vector check. Use a frozen external oracle or an independently
   derived identity. The existing native one-loop reducer/master provider is
   intentionally not a multiloop reduction engine. Do not add another reducer.
3. **Retain a distinct convergence/error-calibration gate for difficult cases.**
   Native Havana assigns complete batches to each sector and requires at least
   two independent batches per sector before reporting uncertainty. This fixes
   the old zero-hit-sector failure structurally. The 32-seed cube-product test
   checks the batch-error normalization, not heavy-tailed multiloop reliability.
   Likewise, completion of a democratic QMC allocation is a coverage property,
   not an empirical convergence certificate. Use several independent seeds and
   nested lattice sizes on selected difficult fixtures, inspect complete-shift
   vectors with the public diagnostic accessor, and compare against a reference
   with known normalization and uncertainty. The historical double-box target
   with null errors cannot certify a pull or act as exact truth.
4. **Boundary growth analysis is still absent.** Current diagnostics enumerate
   bounded face patterns and report finiteness, precision rescue and maximum
   absolute output. They do not estimate componentwise growth between distances,
   scale a growth threshold by codimension, distinguish training-envelope growth,
   or record scaled-distance retries. Finiteness of sampled values is weaker than
   this old diagnostic, and neither is a proof of integrability. If restored,
   implement typed library results over existing evaluations, preserve failures
   from every attempt, and label sampled growth as a diagnostic. Do not introduce
   another symbolic asymptotic engine or silently replace a failed scan with a
   passing farther/closer scan.
5. **Four cheap scientific gaps were closed after this audit.**
   A native timelike triangle/box rejection through the graph entry point; the
   exact original signed/scaled prefactor `-gamma(3+2*eps)` through order four;
   a highest-order request below zero; and a forced MPFR replay of a real native
   rank-two numerator integral would target distinct composition risks. Existing
   native series, graph admission and evaluator APIs implement these operations;
   tests should use those APIs, not new algebra or implementation-call counters.
   All four now pass in `tests/regression_gaps.rs`; see the resolution below.
6. **Some useful reporting remains missing.** Public sector status currently has
   counts/timing, not additive per-sector coefficient estimates. CLI `inspect`
   reports a density or artifact summary but does not yet expose the retained
   chart/pullback metadata. There is no saved-result viewer or typed conversion
   of a completed integration result into an explicitly qualified reference.
   Preserve these as reporting/API gaps; a portable *kernel* artifact is not a
   saved *numerical result*.

## Evidence key

All test paths below are relative to the repository. These are concrete existing
tests, not proposals to reproduce historical private helper layouts.

- **G:** `crates/fastsecdec/tests/generation.rs`:
  `massless_projective_triangle_matches_double_pole_gamma_identity`,
  `massless_projective_bubble_matches_gamma_identity`,
  `full_double_pole_vector_retains_boundary_cancellations`,
  `prefactor_pole_requests_extra_regular_orders`,
  `native_cancellation_avoids_unnecessary_mpfr_rescue`,
  `precision_rescue_resolves_logarithmic_taylor_cancellation`,
  `gamma_prefactor_constants_support_native_conditioning_domain`,
  `portable_artifact_recompiles_and_rejects_modified_content`.
- **S:** `tests/scientific_examples.rs`:
  `native_massless_box_matches_analytic_vector_and_frozen_external_target`,
  `high_order_taylor_cancellation_is_checked_away_from_the_endpoint_threshold`,
  `three_axis_subtraction_preserves_the_complete_meromorphic_polynomial_integral`;
  `tests/subtraction_strategies.rs`:
  `ibp_and_taylor_have_equal_complete_integrals_and_correct_boundary_terms`,
  `a_nonnegative_integer_factor_remains_a_regular_polynomial_weight`,
  `large_regular_polynomial_powers_keep_a_compact_factored_kernel`.
- **M:** `tests/generation_metadata.rs`:
  `retained_maps_are_the_actual_pre_subtraction_pullback_and_keep_gauge_meaning`,
  `assessments_and_exact_chart_associations_survive_portable_roundtrip`,
  `legacy_identity_is_preserved_and_resigned_semantic_tampering_is_rejected`;
  `tests/independent_symmetry.rs` verifies complete-density multiplicity, an
  asymmetric numerator and opposite/unequal weights.
- **N:** `tests/native_input.rs`, especially
  `hepkit_symanzik_and_measure_have_the_expected_signs`,
  `raised_powers_use_stable_edges_and_gamma_normalization`,
  `parameter_binding_precedes_native_scalelessness_and_updates_kinematics`;
  `tests/example_inputs.rs`, especially
  `numerator_fixtures_preserve_reference_denominators_in_native_routing` and
  `native_double_box_matches_the_independent_direct_polynomial_fixture`.
- **H:** `tests/hepkit_one_loop.rs` compares eleven scalar master points through
  native Rust OneLoopMaster, with the explicit Gamma/scale normalization;
  `tests/hepkit_numerator_reduction.rs` compares native one-loop rank-one,
  rank-two, rank-five and Gram-degenerate cases through the existing native
  reducer/master provider. The internal test
  `parametric::numerator::tests::two_loop_mixed_moment_matches_the_product_of_lorentz_moments`
  is an exact separable two-loop identity.
- **Q:** `tests/runtime_qmc.rs`:
  `callback_observes_periodization_weight_applied_exactly_once`,
  `already_weighted_callback_is_accumulated_without_another_jacobian`,
  `full_support_periodized_vector_matches_analytic_cube_integral`,
  `democratic_errors_wait_for_common_full_sector_coverage`,
  `cross_sector_and_exact_cancellations_retain_small_contributions`,
  `absolute_shift_diagnostics_do_not_erase_centered_uncertainty`, and adaptive
  pilot/production/covariance tests.
- **MC:** `tests/runtime_mc.rs`:
  `havana_waits_for_every_sector_before_reporting_uncertainty`,
  `havana_pilot_adaptation_is_excluded_from_production`,
  `havana_batch_uncertainty_is_calibrated_across_independent_seeds`,
  `havana_complete_vectors_match_cube_products_and_covariance`. Native Havana
  configuration rejects fewer than two points per batch or two batches.
- **P:** `tests/weighted_replay.rs`, `tests/independent_weighted.rs` and
  `tests/complex_kernel.rs`, including native multiprecision recovery of complete
  weighted vectors, the asymmetric real/imaginary range-loss regression,
  `gamma_constants_and_complex_boundary_rescue_remain_numeric_on_workers`, and
  `complete_complex_laurent_layout_survives_portable_roundtrip`.
- **C:** `crates/fastsecdec-cli/tests/cli.rs`:
  `every_shipped_run_card_loads_through_the_native_cli`,
  `portable_generation_integration_resume_and_json_errors`,
  `plain_mode_sigint_saves_a_resumable_checkpoint`,
  `transcendental_laurent_artifact_loads_in_a_fresh_process`; driver tests cover
  partial coverage, changed worker count, failed/duplicate submission, pilot
  cancellation and checkpointed weighted maxima. CLI input's
  `native_model_cached_dependents_follow_exact_overrides_and_restrictions`
  preserves exact `pi` through parameter overrides.
- **R:** `tests/reference.rs`:
  `aligns_sparse_complex_keys_without_inventing_missing_zeros`,
  `unknown_errors_and_explicit_exact_zero_have_distinct_meanings`,
  `eligibility_records_coverage_evidence_and_independence_without_overriding_identity`,
  `historical_targets_preserve_unknown_errors_and_unverified_provenance`,
  versioned-envelope and invalid/overflow cases. CLI `tests/reference.rs` covers
  card-relative and command-relative selection, preflight, reference overrides,
  uncertainty/provenance retention and unchanged estimates/checkpoint identity.

## Proposed changes for every remaining row

`I:<line>` denotes `test_integrals.py:<line>` and `B:<line>` denotes
`test_boundary_test.py:<line>`. The original test name and location remain in the
matrix. “Retired” below removes only the named legacy requirement; its scientific
replacement or still-open acceptance gate is recorded explicitly.

### Boundary diagnostics and presentation

| Rows | Proposed status | Replacement evidence or remaining requirement |
|---|---|---|
| B:63 | Pending | Codimension-aware measured growth thresholds are not implemented; existing face enumeration/finiteness cannot cover this. |
| B:109 | Pending | No growth classifier or separate training-growth observation exists. A future classifier must use physical Laurent components and not fail solely on a training envelope. |
| B:209 | Pending | No scaled-distance retry history or policy; existing explicit distance selection and cancellation are only prerequisites. |
| I:33 | Covered | Production source uses native exact Symbolica constants, native series and coefficient conversion at requested precision; G/P exercise Gamma constants and MPFR. The audit found no hard-coded Euler/pi decimal constants in production FastSecDec source. Do not copy the Python regex test over unrelated Rust constants. |
| I:254, I:352, I:364 | Retired | Exact old text widths, ellipsis strings and manual ANSI padding are private formatter mechanics. Ratatui owns styled-cell clipping/layout; native serializable status and C own clean noninteractive output. Keep a visual review of terminal rendering, not tests that mirror character-padding code. |
| I:390 | Retired | No Python child-process initializer. C's real SIGINT test proves the native CLI stops and saves production progress; caller-owned threads have explicit cancellation. |
| I:4240 | Covered | Q's snapshots/coverage checks and C's status stream, partial-run and final-report checks expose accepted aggregate observations through the caller-owned session. There is no library-owned iteration callback to port. |

### Direct evaluation and obsolete intermediate layouts

| Rows | Proposed status | Replacement evidence or remaining requirement |
|---|---|---|
| I:697 | Retired | The user explicitly selected complete Laurent vectors on full sector support. Support-resolved component splitting and reconstruction are removed. G's triangle/cancellation identities and Q's full-support analytic vector test retain the required complete sum. |
| I:728 | Covered | Q verifies Korobov coordinates/Jacobian and both callback contracts apply the weight exactly once; P verifies fused MPFR weighting. There is no requirement that the Jacobian be compiled into a separate support-component JIT kernel. |
| I:1281 | Retired | Tests legacy `asm-o1/o3` and `cxx-o1/o3` dictionaries. Only portable SymJIT O2 is selected/validated in native kernels and artifacts; G/H/S test actual values. |
| I:1545 | Retired | No portable cache of topology-independent projector formulas. Native opaque Laurent templates and whole-kernel content identity provide reuse; G/M/C protect expression/metadata isolation. |
| I:2066 | Retired | No regular-Taylor request scheduler. Finite endpoint powers bypass subtraction in `generation/subtraction.rs`; S verifies finite regular factors and positive powers survive evaluation. |
| I:2083 | Retired | No legacy low-signature cache key. M verifies exact simultaneous coordinate permutation of complete densities and multiplicity. Cache-key equality is not a scientific output. |
| I:2143, I:2186, I:3342 | Retired | Legacy cold-formula volume/axis/pregeneration guards are absent. Native factored expressions, subtraction degree/term limits, decomposition limits and cancellation are the resource contract; S protects compact high-degree factors. This does not waive hard-fixture performance acceptance. |
| I:2284 | Retired | No Python sparse multi-index-set cache. Native Symbolica owns series/truncation and canonical Atoms; no standalone equivalent cache helper is needed. |
| I:2582, I:2658 | Retired | No worker-time endpoint G cache or projector-plan cache. Direct whole-vector kernels replace those calls. S checks independent Taylor/IBP complete integrals; P checks complete-vector rescue rather than reproducing cache entries. |
| I:2807 | Covered | Native mapping uses exact integer powers and native signed polynomial valuation, not floating `exp(log(monomial))` inversion. G's exact near-endpoint cancellation and S's multiaxis/high-degree identities check the resulting behavior. |
| I:2842, I:2914 | Retired | No child-projector Taylor envelope or signature batching. S's intersecting three-axis meromorphic identity and Taylor/IBP comparison cover retained boundary terms. |
| I:3019, I:3047, I:3082 | Retired | Paired derivative-map contexts and special derivative-output evaluators were execution optimizations for the old indirect backend. Native Atom substitution/derivatives build direct expressions; G/S/M verify their composition. |
| I:3109 | Covered | M verifies the actual native coordinate pullback against the retained density, while S independently tests nontrivial differentiated regular factors and intersecting subtractions. No Python chain-rule formula implementation remains. |
| I:3423 | Retired | No dualized-topology alternate backend. Scientific derivative equivalence is covered by S's independent analytic integrals and native Gaussian tests, not comparison of removed evaluators. |
| I:4765 | Retired | The legacy preset forces complex eager mode to avoid an old evaluator problem. Native O2 has scalar/complex and MPFR range/conditioning tests P/G/H; full double-box convergence remains open under I:6113 and the main plan, not a reason to restore a backend selector. |
| I:5801 | Covered | P exercises constant and nonconstant complete complex outputs, native MPFR conditioning, worker cloning and rescue without a dual-input layout. |
| I:5876 | Covered | P/S force real and complex high-order boundary recovery using native multiprecision coordinate arrays. Worker rescue never creates the old double-precision derivative-input matrix. |
| I:5939 | Covered | Geometry exponents and generated coordinate/Jacobian expressions remain native exact integers; M and sectors' `map_validation` tests check their identities. No decimal-coordinate-power rendering is used. |
| I:5988 | Covered | S/G provide independent toy-integral and multiaxis meromorphic identities. Equality to a second legacy formula builder is not needed for the single native direct implementation. |
| I:6045, I:6076 | Covered | S's native graph box and G/H's triangle compare full Laurent vectors with analytic/native-master references. Their evidence is stronger than pointwise agreement of two old builders sharing inputs. |
| I:6113 | Partial | Direct double-box generation, numerical boundary rescue and an independently integrated leading-pole identity are recorded. The completed 64-shift, 6.68-million-evaluation run has no evaluation failures and a leading coefficient -0.000181 +/- 0.000589, consistent with exact zero; higher coefficients remain uncertified. Complete-vector convergence/independent reference acceptance is still open. Do not mark covered merely because legacy projector caches were removed. |
| I:6154 | Retired | Tests only the relative size of two old projector/dual signatures. Keep complete-vector performance gates, not these internal layout counts. |

### Native inputs and numerical composition

| Rows | Proposed status | Replacement evidence or remaining requirement |
|---|---|---|
| I:1059 | Covered | C resolves all shipped TOML graph/model/polynomial inputs from another working directory and preserves explicit integration overrides on resume. R adds relative reference selection/override evidence; YAML and old backend flags are retired. |
| I:4489 | Covered | H compares native triangle generation/O2/QMC against the Rust master provider, and G gives its analytic complete vector. Fixed native Kuo/O2 settings replace the old `auto` eager/complex selection. Default-card loading is C; difficult-fixture convergence remains separate. |
| I:4809 | Covered | `regression_gaps::native_timelike_massless_triangle_and_box_reject_even_with_an_assertion` goes through native graph/kinematics/parametrization and verifies `ComplexBranch` rejection with and without an assertion. Here F is uniformly negative; this is not an interior sign-change witness. |
| I:4831 | Covered | M verifies exact coordinate/measure pullbacks; N verifies native U/F/dimension/power normalization; G/H verify resulting triangle poles. Old sector numbering and a particular two-axis endpoint ordering are not invariants. |
| I:4846 | Retired | Requiring a separate kinematics YAML is obsolete: native HEPKit Kinematics and in-card exact scalar products are authoritative. Native unresolved/unsupported inputs must still fail, and N covers their admission. |
| I:4856 | Partial | Native `inspect --expressions` exposes the complete density and topology counts; retained chart/valuation/measure metadata exist in M. CLI inspection does not yet display that sector metadata/schema. |
| I:4881 | Retired | The old test requires failure without pySecDec. Native sector generation intentionally works without it; actual native geometry/scientific tests replace this placeholder. |
| I:4900 | Partial | Missing files already return ordinary input errors, and C tests changed/missing input handling. No focused missing-graph-path regression was found. Requiring a `.dot` suffix is obsolete: content is passed to HEPKit's parser. |
| I:4944 | Covered | Powers are typed `u32` in the public native override/card contract; N rejects zero and unknown edge IDs. Negative, fractional and symbolic values cannot enter that typed map. Do not introduce a separate DOT power parser or require legacy `power`/`pow` attributes. |
| I:5069 | Covered | The actual original parametrization contains only one-loop triangle-rank-one and box-rank-two cases. H now covers both complete numerical vectors independently. The previous matrix note incorrectly makes this row depend on a multiloop fixture; retain the multiloop gap under I:5096 and the example acceptance checklist. |
| I:5096 | Partial | Exact native two-loop mixed Gaussian moments pass, but only for a separable two-tadpole family. The original coupled sunset family and external-dot term are not cross-checked. See priority 2. |
| I:5172 | Covered | `regression_gaps::native_rank_two_numerator_replay_preserves_the_complete_real_laurent_vector` compares every native rank-two box sector's complete real vector with forced two-precision MPFR replay; H separately supplies the independent integrated numerator reference. |
| I:5216 | Retired | The optional pySecDec numerator backend has been removed. H's independent native one-loop reducer/master comparisons retain its scientific purpose; I:5096 retains the coupled multiloop gap. |
| I:5276 | Covered | `regression_gaps::signed_scaled_gamma_prefactor_keeps_all_coefficients_through_order_four` verifies generated O2 output against the five frozen original values at two coordinates. The original case is **`-gamma(3+2*eps)` through order four**, not a negative Gamma slope. |
| I:5349 | Covered | Native output keys are actual Laurent orders plus real/imaginary components. G's prefactor-pole extension and P's portable complex layout validate keys after convolution; C renders those keys directly. No selectable display-prefactor convention exists. |
| I:5378 | Covered | CLI expressions parse as native Atoms, resolve native exact parameter/kinematic substitutions, and preserve `pi` in C's cached-model override test. N checks scalar-binding order. No YAML-specific evaluator is needed. |
| I:5389 | Retired | The Python comparison helper is absent; C/R cover native run-card and reference path resolution. Kinematic comparison evidence is explicit rather than guessed from a sibling YAML path. |
| I:5400 | Covered | N/H/G cover native triangle topology/normalization and its full analytic/master Laurent vector. Exact pySecDec sector counts and particular monomial lists are intentionally not required. |
| I:5426 | Covered | S's native graph box full vector and N's native Symanzik checks cover the polynomial/sign convention. Exact reference sector count is not required. |
| I:5461 | Partial | N/C prove topology/routing/loading for all six named multiloop fixtures. Their full generation, endpoint regularity and numerical vectors remain unvalidated. |
| I:5507 | Pending | No completed independent full numerical comparison for native `kite_2loop`/`self_energy_3loop`. External oracle execution may be a separate validation job; it must not become a production dependency. |
| I:5752 | Covered | S's three-axis test has a nonconstant regular polynomial factor, negative regulator slopes and exact complete coefficients under both Taylor and IBP. This directly covers differentiation of regular factors as well as monomials. |
| I:6326 | Covered | `regression_gaps::negative_highest_order_integrates_only_the_native_triangle_poles` requests `max_order=-1` through native graph generation, compilation and QMC, and matches the two analytic pole coefficients. A single old training-index selector is obsolete because native training uses a whole-vector envelope. |

### Artifacts, references and statistical reporting

| Rows | Proposed status | Replacement evidence or remaining requirement |
|---|---|---|
| I:3233 | Covered | G/C and `artifact_process::gamma_artifact_loads_in_a_fresh_process` rebuild portable native expressions into O2 evaluators without symbolic generation. Old regular-Taylor sidecar/manifest formats are retired. |
| I:3568 | Covered | M round-trips direct-domain maps, valuations, Jacobians, dimensions and associations; G/P/C round-trip exact/coefficient expressions and O2 policy. N independently equates the shipped graph/direct U/F fixture. A separate old prepared-U/F bundle format is unnecessary. |
| I:3749 | Retired | Portable native artifacts contain required expressions and validated metadata, not external evaluator binary files. Deserialization requires those fields; G/M test malformed/identity-modified payload rejection. There is no missing external JIT sidecar to test. |
| I:4627 | Covered | R preserves missing estimate/reference keys and displays them explicitly, with no invented pull. The CLI delegates its report and unavailable states to the same native API. |
| I:4784 | Covered | R combines independent known standard errors with `hypot`, rejects nonfinite derived values, and keeps unknown/correlated errors unavailable. |
| I:6278 | Retired | Free positional real/imag pairs and zero-filling are removed deliberately. R's typed keyed references reject malformed shapes/duplicates and keep missing coefficients explicit. Preserve valid numeric input through the versioned native envelope. |
| I:6309, I:6590 | Covered | R aligns the union by explicit `(order, component)` keys, including shuffled/noncontiguous poles. Historical zero-filling is intentionally replaced by missing rows; a cancelled/absent coefficient is not inferred to be exactly zero. |
| I:6351 | Partial | Q/MC test correct additive total means, exact offsets and cross-sector covariance. `SectorSnapshot` currently exposes counts/timing only; no public per-sector coefficient-estimate report exists for the old scientific attribution view. |
| I:6376 | Partial | Native tasks and worker contexts use explicit validated sector IDs, with duplicate/foreign-result rejection. There is no CLI sector-subset integration mode with an explicitly qualified partial-total result. Such a mode must not claim full-integral convergence or lose grouped-chart multiplicity/exact contributions. |
| I:6401 | Partial | C/R serialize final result/reference data and preserve comparison provenance; versioned reference documents round-trip. There is no saved-integration-result reader/viewer with additive sector sorting, nor automatic result-to-reference conversion. |
| I:6481, I:6524 | Partial | Native artifacts retain selected reference steering and R/CLI round-trip the selected reference's provenance/error meaning. They do not import a saved integration report and choose its nested reference. Any future adapter should require an explicit selection and never silently promote the estimate. Old pySecDec-specific nesting is not required. |
| I:6601 | Pending | `inspect` views input/kernel artifacts, not saved numerical result files. A result-only reader/viewer should avoid graph generation and JIT compilation; no current equivalent exists. |
| I:6621 | Retired | The exact `32 * 616` autoscale rule compensates for random discrete-sector hits in the old sampler. Native Havana plans independent batches for every sector, with explicit work limits and no library-owned schedule. MC/Q enforce complete sector support. Difficult-integrand calibration remains priority 3. |
| I:6643 | Covered | MC withholds uncertainty until every stochastic sector has two complete production batches; Q requires common complete-shift support. Incomplete allocations cannot meet tolerance. Thus a zero-hit sector cannot be assigned a reported full-integral error. Legacy warn/auto switch names are unnecessary. |
| I:6673 | Partial | R blocks eligibility for incomplete production/unverified references and never imports historical status prose as trust. General saved-result-to-reference conversion is absent, so no native regression yet exercises rejection of an unreliable saved numerical result as a target. Preserve that safety requirement when adding the adapter. |
| I:6700 | Retired | No statistical-safety-off override exists. Native estimates retain `production_complete`; R's eligibility records incomplete coverage even when diagnostic differences are displayed. Do not add a flag that upgrades coverage to scientific certification. |

## Ecosystem reuse and next audit boundary

Existing owners remain appropriate: HEPKit/FeynKit for physical graphs and
kinematics; native scalar masters and one-loop reduction for one-loop numerical
oracles; Symbolica for exact series, substitution, derivatives, rational
integration, constants and multiprecision evaluators; Numerica for MC/QMC samples,
covariance and centered sums; Graphica for candidate symmetry canonization.
Neither the missing multiloop oracle nor boundary/reporting gaps justify a new
CAS, graph parser, tensor reducer, covariance engine or general asymptotic solver.

At the next milestone, update the matrix using these recommendations only after
the connected source gate passes. Keep unverified numerical fixtures and
performance results separate from retired private APIs. The next recurring
integration audit should check that any new per-sector/result/boundary status is
library-owned, preserves complete-vector covariance and partial-coverage meaning,
and can be consumed by the future HEPKit bridge without JSON or DOT detours.

## Independent CLI reference wrapper review

Reviewed `crates/fastsecdec-cli/src/reference.rs`, `main.rs`, `generate.rs`,
`artifact.rs`, `input.rs` and their focused tests. The wrapper delegates parsing
and comparison to the native library, performs reference preflight before graph
loading or portable kernel compilation, checks optional kernel identity, and
keeps comparison steering outside numerical checkpoint/scientific identity.
Missing integration coverage is represented by a distinct waiting report with
the native reference/context and no fabricated rows.

The coordinator found and the owner fixed two issues before this review closed:
evidence for a different overridden target must be cleared; run-card semantic
fingerprints must use native TOML serialization rather than lossy TOML-to-JSON
conversion. The current source preserves nonfinite TOML float distinctions and
date versus string types, retains unknown root fields, removes only root
`[reference]`, and preserves byte-hash semantics for old source records. Tests
cover these boundaries and reference/checkpoint identity across working
directories and worker counts. No additional actionable wrapper finding was
identified. The library display was subsequently changed to readable reasons and
aligned numeric columns at the coordinator's request; numerical contracts did
not change. The final nine library and 22 CLI tests, formatting and Clippy passed
with these edits before milestone `b94053f` was committed and pushed.

## Targeted scientific gap resolution

After the audit, the coordinator authorized four focused regressions in
`crates/fastsecdec/tests/regression_gaps.rs`. All four passed in 0.24 seconds
after compilation (`output/regression-gap-tests.log`). No production changes
were necessary. The timelike test initially requested the wrong error variant:
uniformly negative F with a regulator-dependent power correctly returns
`ComplexBranch`, rather than `Threshold` for an interior zero/sign change. The
test now checks that precise scientific distinction. The rank-two test compares
the entire native Gaussian-generated vector at a regular point with a forced
256-/512-bit replay for every sector, while the existing master/reduction tests
remain the independent integral reference. No new series, tensor reduction or
statistical algorithm was introduced.
