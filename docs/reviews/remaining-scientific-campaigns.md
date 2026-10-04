# Remaining scientific example campaigns

This is an execution schedule, not an assertion of completed phase-one parity.
It separates input equivalence, generated complete Laurent vectors, numerical
integration, independent reference evidence and convergence. It was assembled
after the six massive-family comparisons, without rerunning those families or
the passed native one-loop master/reduction suites.

## Shipped card inventory

All 24 cards pass the production CLI's input-loading test, including native DOT,
model/parameter cards, kinematics, Gaussian numerator conversion and external
polynomial-file resolution. Native graph tests additionally check export/reload
and exact propagator/numerator equivalence; the double-box graph and direct
polynomial inputs have an exact U/F and measure-equivalence test. Those input
checks alone do not establish an integrated value.

The added native coupled-sunset card brings the inventory to 24 cards and 17
DOT graphs. Its dedicated native routing, independent density and integrated
coefficient checks pass; the combined milestone gate also checks all-card CLI loading.

`G` means full generation including requested Laurent outputs; `V` means a
complete numerical vector. Native-master/reducer rows use the same graph and
kinematics with the explicitly tested analytic OneLOop normalization conversion.
They are not claims that a serialized CLI artifact was generated from that exact
run card. `Pending` never means a zero integral or an excluded task.

| Native run card | G / V evidence | Independent coefficient evidence | Remaining scientific work |
| --- | --- | --- | --- |
| `bubble` | Both, native B0 and CLI checks | Native B0 at this massless point; three other massive/scaleless/scale points | Formal matched convergence/performance campaign |
| `triangle` | Both, analytic/CLI/native-master suites | Analytic vector, historical external target and native C0 | Formal matched convergence/performance campaign |
| `triangle_massive` | Both at this same massive point in native-master suite | Native C0 | Formal matched convergence/performance campaign |
| `box` | Both, complete-vector suite | Analytic vector, frozen external target and native D0 | Formal matched convergence/performance campaign |
| `box_massive` | Both at the exact card point `s=t=-1`, mass squared 1 | Native D0 complete-vector comparison; additional `s=-1,t=-2` case retained | Formal matched convergence/performance campaign |
| `triangle_numerator` | Both, native reduction/master suite | Native reducer + masters; preserves actual virtualities `{-1,0,-2}` | Convergence; do not substitute the scalar triangle's point |
| `box_numerator` | Both, native reduction/master suite | Native reducer + masters, massless and additional massive point | Convergence |
| `box_rank2_numerator` | Both, native reduction/master suite | Native reducer + masters, including massive and zero-Gram checks | Convergence; forced native MPFR vector also covered |
| `box_high_rank_numerator` | Both, native reduction/master suite; complete-vector support-cache equivalence verified | Native reducer + masters for all three coefficients | Matched performance and convergence |
| `double_box` | Both; complete 64-shift five-coefficient run | Exact native rational proof of the leading zero only; higher target errors unavailable | Independent higher-coefficient reference and matched convergence |
| `double_box_from_uf` | Exact equality to the graph density; generation/integration family above | Same leading-pole proof applies to the identical density | Optional fresh CLI transport closure; no duplicate scientific integral campaign |
| `triple_box` | Original and projected-family bounded trials timed out during Laurent extraction; no artifact or V | No frozen independently certified full vector | Compact native-series composition and guarded depth selection before repeating full generation |
| `triple_box_offshell` | Both; 2496 charts, 1182 kernels and complete 1024-by-eight full-vector allocation; projected eight-propagator family also complete | Exact native domain/normalization and repeated-propagator identity; no independent integrated vector | Independent full-vector reference, convergence and matched projection cost |
| `triple_box_offshell_rank2_numerator` | Both original and projected families; each accepts all 9,682,944 evaluations with zero failures | Exact numerator/routing, family identity and coupled-Gaussian control; no independent integrated vector | Independent full-vector reference, convergence and matched projection cost |
| `kite_2loop` | Both; fixed Stage A work scaling with two rules and three seeds | Frozen independently generated pySecDec finite result; all Stage A comparisons below the investigation threshold | Prespecified higher-work block and matched performance |
| `self_energy_3loop` | Both; fixed Stage A work scaling with two rules and three seeds | Frozen independently generated pySecDec finite result; all Stage A comparisons below the investigation threshold | Higher-work block; Kuo uncertainty plateau; matched performance |
| `three_point_2loop` | Both; fixed Stage A work scaling with two rules and three seeds | Frozen independently generated pySecDec finite result; all Stage A comparisons below the investigation threshold | Prespecified higher-work block and matched performance |
| `three_point_2loop_6line` | Both; original diagnostics and fixed Stage A with new seeds | Frozen independently generated pySecDec finite result; independent seeds reproduce the Kuo plateau and HKKN improvement | Prespecified higher-point and 64-shift controls; retain all observations before default policy |
| `three_point_3loop` | Both; fixed Stage A work scaling with two rules and three seeds | Frozen independently generated pySecDec finite result; all Stage A comparisons below the investigation threshold | Higher-work block; Kuo uncertainty plateau; matched performance |
| `three_point_3loop_8line` | Both; fixed Stage A work scaling with two rules and three seeds | Frozen independently generated pySecDec finite result; all Stage A comparisons below the investigation threshold | Higher-work block; uncertainty remains substantial under both rules; matched performance |
| `analytic_endpoint` | Both, independent analytic full-vector and CLI artifact/resume tests | Analytic coefficients through order `eps^1` | Retain as a cheap operational/control fixture |
| `issue_1` | Both; 328 kernels, complete orders `[0,1,2]` and 2,686,976 accepted evaluations without failures | Manual decimal target through `eps^2`, uncertainty unavailable | Independent external reference and prespecified convergence |
| `four_loop_hard` | Both; 2760 physical F-support charts, 699 kernels, orders `[-2,-1,0]` and all 5,726,208 evaluations without failures | Rounded historical full-vector QMC target, uncertified errors | Independent full-orthant reference and prespecified convergence |
| `sunset_2loop_numerator` | Both, complete vector through epsilon one at two spacelike scales | Nine independent external density points and analytic Gamma identity; convergent scalar sign control | Retain as a coupled-Gaussian control for the triple-box numerator |

No example above currently establishes the overall matched performance gate.
The earlier 64-shift double-box diagnostic used SymJIT 2.26.0 and its preserved
artifact; current-backend short scientific gates passed after upgrading to
2.26.4, but that long diagnostic must not be relabelled a current-backend timing.
The six massive comparisons used 2.26.4 and preserve complete native covariance;
the external runs use an independent reference-only dependency stack.

The fixed [Stage A campaign](massive-holdout-stage-a.md) completed all 72 rows
and 223,838,208 accepted evaluations without failures or timeouts. No comparison
crossed the prespecified five-combined-standard-error investigation threshold.
HKKN has lower estimated uncertainty in all six cases at 8192 points, but higher
uncertainty in four at 1024. Complete native shift vectors and cross-rule joint
covariance are retained; these finite samples do not calibrate statistical
coverage or justify a universal default change.

## Additional coupled two-loop numerator regression

The original reference test at `test_integrals.py:5096` is an inline family, not
one of the original 23 shipped cards. It has propagators `k1^2`, `k2^2`,
`(k1+k2+p1)^2`, external `p1^2=-1`, mass zero and numerator
`k1.k2 + 2*k1.p1`. Its comparison checks preliminary Gaussian polynomials at
`x=[0.17,0.29,0.54]` and `eps=[0,0.11,-0.19]`. The existing native separable
two-tadpole moments do not close this coupled-family gap.

The implemented native HEPKit sunset uses the shared model and native momentum
tensors. Exact routing, nine independent full/Gamma-stripped density checks,
the convergent scalar sign control and the complete Laurent vector at two scales
pass. It reuses the current Gaussian construction and native Symbolica series;
there is no alternate parser, graph, integrator or multiloop reducer. The native
one-loop reducer remains outside this two-loop reference path. See the
[campaign](coupled-sunset-numerator.md) and
[independent review](coupled-sunset-independent.md).

## Ordered bounded campaigns

1. **Coupled numerator.** The exact `box_massive` card point is now closed by
   the focused native D0 test, passing in 1.95 seconds; the earlier eleven
   scalar points were not repeated. The native coupled sunset fixture now
   passes all four scientific tests in 1.16 seconds. Its initial scalar-control
   uncertainty missed the predefined precision cap; raising only that control
   from 4096 to 65536 points per shift passed without changing the tolerance.
   Both numerator scales retain 4096 points and 16 shifts. This gate permits the
   coupled triple-box campaign, but does not certify it in advance.
2. **Off-shell triple box, then its rank-two numerator.** Start with the scalar
   off-shell density at masses zero, all four virtualities `-1`, `s=t=-2`.
   Record Gaussian/admission, geometry, mapping, symmetry, subtraction, Laurent,
   compilation and reload separately. Give native generation/compilation a
   cooperative limit plus an external 300-second watchdog for the first trial;
   retain partial progress and peak process memory on timeout. After completion,
   allocate 1024 points and eight shifts to every retained kernel, with a
   180-second numerical watchdog and full-vector/coverage reporting. The
   numerator follows with identical kinematics and its independently checked
   native routing; it is not compared sector-by-sector with the scalar result.
   Both original-graph campaigns now completed within those budgets, retaining
   every order `[-3,-2,-1,0]` and all 1,182 representative kernels. Scalar and
   numerator generation took 51.705 and 55.406 seconds respectively; fresh-process
   integration took 146.205 and 56.313 seconds, including loading, full status
   snapshots and checkpoint I/O. Both complete allocations report zero evaluation
   failures and unmet tolerance. These are preliminary estimates, not isolated
   throughput or independently certified values. The historical topology has
   two pairs of repeated propagators, so it is not the ordinary undotted ladder
   and off-shell kinematics do not imply finiteness. Native `partial_fraction`
   and `sector` APIs exactly reduce it to eight active propagators while retaining
   total power ten; that separate reuse probe does not change either baseline.
   See [the full diagnostics](triple-box-offshell-diagnostics.md).
   Both projected-family trials now also complete, retaining 1,182 kernels and
   the same full coefficient layout in seven rather than nine dimensions.
   Their exact native denominator identity is the equivalence proof; the first
   numerical estimates remain preliminary. The shared seed can correlate the
   two parameterizations, so their errors are not combined as independent.
   Projected generation and artifact sizes are smaller, but different caller
   persistence/status policies and builds preclude a matched speedup claim.
   See [the projected-family evidence](native-family-projection-diagnostics.md).
3. **On-shell triple box and issue-1 orthant.** The on-shell triple box adds
   multiple poles at masses zero, null external legs and `s=t=-1`; require the
   complete requested Laurent vector through zero. Issue-1 instead retains its
   seven-dimensional positive orthant, complete density `F^(eps-2)`, unit
   prefactor and orders through two. Use the same staged 300/180-second first
   watchdogs and 1024-by-eight initial all-kernel allocation, then inspect native
   contributions and failures before increasing work. Historical manual targets
   remain diagnostic rows with unknown uncertainty; no eligible statistical
   pull is assigned to them.
   The first on-shell trial reached 2,112 charts and 1,026 representatives, but
   timed out at the 300-second generation limit plus five-second cancellation
   grace. Completed Laurent work accounted for 282.024 seconds; geometry,
   mapping and symmetry took 1.309, 1.905 and 1.622 seconds. Its last entry callback
   was displayed representative 46 (zero-based index 45), at 288.504 seconds.
   No artifact or usable partial integral was produced. Retain this failure
   while a test-only native-template replay and eight-propagator family probe
   investigate the bottleneck; extending the same run is not the next step.
   Those diagnostics are now available. Native relative-depth selection preserves
   every coefficient of the captured representative while reducing its series
   work from 148.974 to 37.332 seconds; its large final expressions remain.
   The projected eight-propagator on-shell trial also timed out, at 310.289
   seconds, after only 38 completed Laurent phases. It produced no artifact or
   numerical observation. Native series composition before coordinate
   differentiation is therefore being prototyped under exact small-vector and
   native truncation-bound controls. See the
   [depth attribution](native-laurent-depth-probe.md) and
   [projected follow-up](native-family-projection-diagnostics.md).
   Issue-1 completed its first full generation in 3.463 seconds and integration
   process in 4.065 seconds, retaining all three generated coefficients and their
   covariance. It reached the work limit without meeting tolerance. The stored
   historical decimal target remains statistically ineligible because its
   uncertainties are unknown. See the
   [bounded on-shell/orthant evidence](onshell-triple-box-and-issue-one.md).
4. **Hard four-loop positive orthant.** Preserve all nine variables and density
   `U*F^(eps-3)` with unit prefactor. Integer-power negative U is intentional.
   The already passed 3496-map geometry test does not establish successful
   Laurent generation or O2 compilation. First record a bounded 600-second
   generation/compile trial with cancellation and retained stage/memory reports.
   On success, measure a short all-kernel evaluation budget before choosing
   production work; do not silently run only a convenient sector subset.
   Use native caller-owned QMC work/precision/status APIs throughout. A partial
   allocation remains coverage-incomplete and supplies no fabricated total.
   The first complete trial now passes: generation/compilation took 36.755
   process seconds; all 699 kernels completed 1024 points times eight shifts in
   124.493 process seconds including loading. The full three-coefficient vector
   and covariance remain saved, with zero evaluation failures and work-limit
   stopping without convergence. Production decomposes only the singular F
   support (2760 charts); the earlier combined-U/F 3496-map stress test has a
   different workload. The independent-reference and convergence gates remain
   open. See [the full evidence](hard-four-loop-diagnostics.md).

These are first diagnostic budgets, not acceptance limits or permission to
silently omit a hard case. A timeout identifies the stage for evidence-based
native reuse/performance work before repeating it. All large symbolic processes
and timing trials require the existing shared runtime/build coordination.

## Independent references and later convergence

For graph families without native multiloop masters, the existing reference
pySecDec-only bridge is the next independent implementation. Use serial package
generation, explicit normalized measure and the actual native-equivalent
kinematics. The historical off-shell triple-box card warns that package
generation can exceed 30 GiB, so attempt an explicitly bounded reference build
only after native admission succeeds and retain resource failures honestly.
Reference failure does not authorize copying a reducer or calling an internal
finite estimate independent certification.

Issue-1 and the hard density require their explicit orthant-domain reference
inputs and unit prefactors. Do not turn them into graph projective-simplex
integrals. The hard `psd2807` steering variants cover one historical raw sector,
not the full integral; neither matching its integer sector ID nor matching
sector counts is a valid native correspondence. A verified complex O2 or eager
Pathfinder run can provide a secondary comparison where independent pySecDec
generation is impractical, with the same dependency/backend caveats recorded.

After complete-vector correctness, run at least three independent seeds at
matched actual point counts for each selected published lattice/periodization
choice, preserving all trials, common-shift totals and full covariance. Increase
points within native catalogue limits, then independent shifts at the cap.
Separate catalogue comparisons from scalar/JIT rescue changes. Pilot allocations
must be independent of frozen production; no selection of only the smallest
observed error is permitted. Match generation, compilation, reload, numerical
arithmetic and worker settings before making performance claims.

The existing scalar/numerator master suites, six frozen massive references,
analytic endpoint, and exact double-box leading-pole proof remain controls.
Their passed work need not be recreated merely to prepare this schedule.
