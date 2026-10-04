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
| `triple_box` | Full G/V pending; native input checked | No frozen independently certified full vector | Bounded staged full-vector campaign |
| `triple_box_offshell` | Full G/V pending; native input checked | No frozen independently certified full vector | Bounded scalar off-shell campaign before its numerator |
| `triple_box_offshell_rank2_numerator` | Full G/V pending; exact numerator/routing checked | No independent integrated vector; native one-loop reducer is inapplicable | Coupled-Gaussian gate, then bounded full-vector campaign |
| `kite_2loop` | Both, six-case production CLI harness | Frozen independently generated pySecDec finite result | Multi-seed work-scaling and matched performance |
| `self_energy_3loop` | Both, six-case production CLI harness | Frozen independently generated pySecDec finite result | Multi-seed work-scaling and matched performance |
| `three_point_2loop` | Both, six-case production CLI harness | Frozen independently generated pySecDec finite result | Multi-seed work-scaling and matched performance |
| `three_point_2loop_6line` | Both, initial and independent-seed higher-work runs | Frozen independently generated pySecDec finite result | Published-catalogue controlled comparison already isolates rule quality; retain all observations before default policy |
| `three_point_3loop` | Both, six-case production CLI harness | Frozen independently generated pySecDec finite result | Multi-seed work-scaling and matched performance |
| `three_point_3loop_8line` | Both, six-case production CLI harness | Frozen independently generated pySecDec finite result | Multi-seed work-scaling and matched performance |
| `analytic_endpoint` | Both, independent analytic full-vector and CLI artifact/resume tests | Analytic coefficients through order `eps^1` | Retain as a cheap operational/control fixture |
| `issue_1` | Input checked; full G/V pending | Manual decimal target through `eps^2`, uncertainty unavailable | Bounded orthant G/V, independent external reference, then convergence |
| `four_loop_hard` | Input and exact 9D geometry/maps checked; full Laurent G/V pending | Rounded historical full-vector QMC target, uncertified errors | Bounded whole-integrand generation/compilation, all-sector numerical diagnostics, independent reference and convergence |
| `sunset_2loop_numerator` | Both, complete vector through epsilon one at two spacelike scales | Nine independent external density points and analytic Gamma identity; convergent scalar sign control | Retain as a coupled-Gaussian control for the triple-box numerator |

No example above currently establishes the overall matched performance gate.
The earlier 64-shift double-box diagnostic used SymJIT 2.26.0 and its preserved
artifact; current-backend short scientific gates passed after upgrading to
2.26.4, but that long diagnostic must not be relabelled a current-backend timing.
The six massive comparisons used 2.26.4 and preserve complete native covariance;
the external runs use an independent reference-only dependency stack.

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
3. **On-shell triple box and issue-1 orthant.** The on-shell triple box adds
   multiple poles at masses zero, null external legs and `s=t=-1`; require the
   complete requested Laurent vector through zero. Issue-1 instead retains its
   seven-dimensional positive orthant, complete density `F^(eps-2)`, unit
   prefactor and orders through two. Use the same staged 300/180-second first
   watchdogs and 1024-by-eight initial all-kernel allocation, then inspect native
   contributions and failures before increasing work. Historical manual targets
   remain diagnostic rows with unknown uncertainty; no eligible statistical
   pull is assigned to them.
4. **Hard four-loop positive orthant.** Preserve all nine variables and density
   `U*F^(eps-3)` with unit prefactor. Integer-power negative U is intentional.
   The already passed 3496-map geometry test does not establish successful
   Laurent generation or O2 compilation. First record a bounded 600-second
   generation/compile trial with cancellation and retained stage/memory reports.
   On success, measure a short all-kernel evaluation budget before choosing
   production work; do not silently run only a convenient sector subset.
   Use native caller-owned QMC work/precision/status APIs throughout. A partial
   allocation remains coverage-incomplete and supplies no fabricated total.

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
