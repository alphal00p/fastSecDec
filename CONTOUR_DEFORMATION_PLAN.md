# Phase B: causal contour deformation

## 1. Goal, scope and working rules

**Goal statement**

> Implement opt-in contour deformation in FastSecDec, delivering validated fixed-strength deformation first and smooth, causally bounded dynamic deformation second. Preserve endpoint subtraction, analytic branches, complete complex Laurent vectors, portable evaluators and bounded-memory execution. Make runtime causal validation optional in both modes. Optimize generation and especially sampling runtime throughout, validate against independent physical and multiloop references, and complete scientific, performance and HEPKit integration audits before declaring this phase finished.

Upon starting implementation:

- Create and work on the new **`contour_deformation`** branch, overriding the earlier instruction to implement directly on `main`.
- Write this approved plan **verbatim** into `CONTOUR_DEFORMATION_PLAN.md`, reference it from `FIRST_PHASE_PLAN.md`, and assign the goal above before changing implementation code.
- Commit and push validated milestones on the feature branch.
- Delegate mathematical research, generation, runtime implementation, interfaces, performance and independent reviews to separate agents. The coordinating agent reviews their integration and acceptance evidence.

This phase covers **real masses and real physical kinematics**, including complex numerators and amplitudes. Complex masses, GCAD and alternative deformation directions are outside this delivery.

**Symbolica reuse is mandatory for every CAS task.** Triple-check its public API, source/tests and a focused executable probe before implementing an operation. Record the evidence. Implement missing functionality only after explicitly demonstrating its absence; prefer a small owner-library improvement over a parallel algebra implementation. Apply the same discipline to Numerica, HEPKit and Linnet facilities.

Runtime performance is a requirement throughout development. Preserve factored expressions, share computations and measure actual sampling cost before accepting major design choices.

## 2. Mathematical construction and validation policy

### Shared deformation

After sector mapping and endpoint-monomial extraction, designate the residual causal polynomial \(F\) explicitly and retain positive residual factors, including \(U\). Preserve the positive relation between this designated \(F\) and the original causal denominator.

Define

\[
w_i=x_i(1-x_i),\qquad
v_i=w_i\,\partial_iF,\qquad
z_i=x_i-i\lambda v_i.
\]

The scalar \(\lambda\) follows the pySecDec strength convention; it is not literally the displacement’s Euclidean norm. Do not introduce an undocumented normalization of \(F\).

The map preserves coordinate faces, with

\[
\operatorname{Im}F(z)
=-\lambda A+O(\lambda^3),
\qquad
A=\sum_iw_i(\partial_iF)^2\ge0.
\]

Separate causal direction, branch preservation, permitted magnitude and numerical conditioning, taking inspiration from the [LTD construction](https://arxiv.org/html/1912.09291v2) while deriving the bounds for parameter-space polynomials.

### Fixed mode — implement first

Accept any finite **\(\lambda_{\mathrm{fixed}}>0\)** at runtime.

Construct the Jacobian symbolically with Symbolica and optimize it with the complete integrand. When validation is enabled, check causal and positive-factor conditions at the required points and subtraction-face arguments, with preflight homotopy diagnostics.

A failed check reports the sector, coordinates, parameters and offending factor. **Never silently change a fixed user value.**

Finite sampled checks are safeguards comparable to pySecDec’s checks, not a global certificate. [pySecDec documentation](https://secdec.readthedocs.io/en/stable/faq.html)

### Dynamic mode — required second delivery

Use

\[
\lambda(x)=S\,r(x),\qquad 0<S<1,
\]

where \(r(x)\) is a **smooth sufficient causal radius**, rather than the nearest complex zero.

The nearest-zero rule is unsuitable: a positive real root generally does not exist, and taking the modulus of a complex root can make the deformation vanish at a physical pole.

The construction must control the entire homotopy \(x-itv(x)\), \(0<t\le\lambda(x)\). Its mathematical validity must follow from explicit bounds, independently of presampling or optional runtime checks.

**Reference construction.** For odd \(k\ge3\), define

\[
T_k=D^kF(x)[v^{k-2},\,\cdot,\,\cdot],\qquad
B_k=\frac{\sum_{ij}w_iw_j(T_k)_{ij}^2}{(k!)^2}.
\]

These smooth polynomial quantities bound higher imaginary terms relative to \(A\), without division by \(A\). Obtain them through Symbolica’s directional series of the Hessian, holding the direction fixed during the auxiliary expansion.

For each positive residual \(U\), obtain its exact even ray coefficients \(d_k\), with \(C_k=d_k^2/U(x)^2\). Establish positivity of the residual on the closed cube through its native polynomial representation; never substitute an arbitrary positive floor.

For \(L=\lambda_{\mathrm{cap}}>0\), construct

\[
H(x,r)=
\frac{r^2}{L^2}
+\frac{r^2\|v\|^2}{R^2}
+m_F\sum_{\text{odd }k\ge3}B_kr^{2k-2}
+\sum_Um_U\sum_{\text{even }k\ge2}C_kr^{2k}.
\]

The structural counts \(m_F,m_U\) remain fixed. Choose the unique positive root of \(H=1\).

The cap and displacement constraints are smooth, with no hard minimum. Initially use \(L=1\) and \(R=1\), exposing both positive limits in native settings. The displacement constraint prevents unnecessarily enormous excursions from dimensionful gradients. These limits affect efficiency, while the causal bounds remain valid for any positive choices.

Nonnegative coefficients give a unique simple root, \(r\le L\), and \(rH_r\ge2\). The safety fraction \(S<1\) preserves the required branches throughout the homotopy.

**Tighter production construction.** Implement a sign-aware refinement that bounds only higher terms opposing causality:

- Form the weighted symmetric matrices
  \[
  R_k=\frac{(-1)^{(k+1)/2}L^{k-1}}{k!}\sqrt W\,T_k\sqrt W.
  \]
- Bound their largest eigenvalue through trace and squared-norm identities, without eigensolves or explicit square roots of \(W\).
- Smooth the positive part using a fixed dimensionless regularity parameter, initially \(10^{-3}\), evaluated through cancellation-resistant expressions.
- Treat harmful even \(U\) coefficients similarly.
- Construct nonnegative envelopes \(E_F,E_U\) and solve
  \[
  u^2+\frac{L^2u^2\|v\|^2}{R^2}
  +E_F(u)^2+\sum_UE_U(u)^2=1,
  \qquad \lambda=SLu.
  \]

Retain the polynomial construction as an independent correctness baseline. Benchmark the sign-aware refinement’s complete cost, including higher derivatives. Larger permitted displacement is a useful diagnostic, but does not by itself establish improved convergence.

These bounds are the proposed parameter-space construction developed during planning, not algorithms attributed to pySecDec or the LTD paper.

### Derivatives, branches and exceptional points

For dynamic strength,

\[
J_{ij}=\delta_{ij}
-i\left[\lambda\,\partial_jv_i+v_i\,\partial_j\lambda\right].
\]

Use implicit derivative hooks and Symbolica’s native dual machinery for all derivatives required by subtraction. Do not differentiate solver iteration decisions or omit derivatives of the smooth cap.

Define one radius function per sector. Every subtraction face restricts that same function, including its dimension, structural counts and caps.

Preserve factorwise analytic continuation:

- Keep endpoint monomial phases separate from residual \(F/U\) logarithms.
- Do not replace their continued logarithmic sum with the principal logarithm of the product.
- Explicitly implement the causal lower-lip value for negative-real \(F\) with zero imaginary part.
- Treat stationary nonzero \(F\) correctly. A zero with vanishing weighted gradient requires an unresolved-deformation diagnostic, not an automatic zero or an unsupported claim of a Landau pinch.
- Do not classify a zero complex Jacobian alone as a pole crossing.

### Optional runtime causal validation — both modes

Expose a separate runtime policy:

| Policy | Behavior |
|---|---|
| `always` — default | Validate the pilot/preflight and production evaluations. |
| `pilot` | Validate a configurable pilot, then perform production without causal-validation overhead. |
| `off` | Skip optional runtime causal validation entirely. |

The pilot must exercise sector interiors and subtraction-face evaluations. Use existing sampling facilities with separate validation work identities; pilot observations never enter production statistics. Record its settings, checked coverage and outcome.

Changing kinematics, the deformation prescription, \(S\), \(\lambda\), or cap settings invalidates previous pilot evidence. Changing only the validation policy does not change the mathematical integrand or invalidate accepted integration statistics.

When checks are enabled, use native certified ball arithmetic for algebraic safety inequalities and numerical root enclosures, with precision escalation when needed. Ordinary error tracking is not a certificate. When checks are disabled, **do not secretly retain these expensive per-sample validations**.

Essential solver termination and nonfinite-value handling remain active. Disabling validation does not authorize silently discarded samples, fabricated zeros or emergency changes to \(\lambda(x)\).

The dynamic construction retains its exact-arithmetic causal proof with checks disabled. Results must distinguish that mathematical guarantee from whether production evaluations received numerical validation; a successful finite pilot is not a global floating-point certificate.

## 3. Generation, runtime and public interfaces

Use the sequence

**sector extraction → deformation of the smooth density → Taylor/IBP subtraction → Laurent construction → evaluator optimization.**

Keep the real endpoint powers and insert

\[
\det J\prod_i(z_i/x_i)^{a_i+b_i\epsilon}
\]

into the smooth remainder before subtraction. Represent \(z_i/x_i\) analytically to avoid endpoint \(0/0\). Deform numerators and all affected factors consistently.

Update symbolic and numerical-dual generation together, including monomial-specific jet masks. Local-strength derivatives must participate in the Jacobian and subtraction. [Local deformation and Jacobian](https://arxiv.org/html/2112.09145v2)

Proposed CLI:

```bash
fastsecdec generate run.toml --contour

fastsecdec integrate integral.fsd \
  --contour fixed --lambda 0.1

fastsecdec integrate integral.fsd \
  --contour dynamical=0.8 --lambda-cap 1 \
  --contour-validation pilot
```

- Deformation remains opt-in; runtime validation defaults to `always`.
- `run` forwards generation capability and runtime settings.
- Mirror settings in TOML, native types and thin HEPKit wrappers.
- Rebinding strengths, caps or validation policy must not repeat symbolic generation or Horner/CPE optimization.
- Reject deformation requests for artifacts lacking the required capability.

Extend universal artifacts with causal-factor identity, deformation recipes, branch semantics, derivative requirements and evaluator programs. Preserve selective sector loading and every normal/serial execution combination.

Korobov remains the outer real transformation:

\[
y=T(q),\qquad z=z(y),\qquad
J_{\mathrm{total}}=\det J_z(y)\prod_iT_i'(q_i).
\]

Multiply each Jacobian once; never divide by a vanishing Korobov derivative.

Include mathematical contour settings and recipe version in checkpoint compatibility. Record validation policy and pilot provenance separately, permitting checked-pilot-to-unchecked-production workflows without resetting statistics.

Deformation must be deterministic and consume no production sampling RNG streams. Preserve caller-owned execution, full covariance and ordinary discrete MC support.

Extend status snapshots and inspection with maps, Jacobians, validation policy, pilot coverage, strength/displacement statistics, root iterations, precision rescues and evaluator sizes. Aggregate diagnostics at existing update boundaries.

Keep substantive code in Rust/FastSecDec. Community remains registration/reexports/stubs only. Validate native, portable and HEPKit/WASM execution.

## 4. Implementation milestones and runtime optimization

1. **Foundation and theory audit.** Record proofs, counterexamples, branch requirements and reuse evidence. Preserve the undeformed fast path without contour overhead.

2. **Fixed-mode milestone.** Complete deformation, subtraction, branches, optional validation, runtime binding and artifact restoration. Pass analytic threshold tests before implementing dynamic production integration.

3. **Dynamic correctness milestone.** Implement the polynomial envelope, root callback, complete implicit jets and optional certified checks. Add and independently validate the sign-aware refinement.

4. **Performance milestone.** Optimize throughout:
   - Share \(F/U\), gradients, directional coefficients, roots and Jacobians across Laurent outputs.
   - Evaluate one scalar root per distinct required coordinate/face request.
   - Use Symbolica-derived closed forms for low-degree envelopes.
   - Avoid evaluator construction and generic all-complex-root searches in the sampling hot path.
   - If efficient higher-degree refinement requires a missing native API, document the triple-check evidence and make a narrow owner-library improvement.
   - Compare determinant/solve representations, aliases and inlining through measured costs.
   - Measure validation overhead separately and verify that unchecked production removes it.
   - Retain SymJIT O2, precision rescue, portable restoration and bounded residency.

5. **Delivery audit.** Independently review mathematics, branches/subtraction, numerical validation, performance, checkpoints/seeds and HEPKit reuse. Commit and push accepted milestones; complete the goal only after required gates pass.

Planning probes already verified Symbolica implicit higher jets, complex SymJIT callbacks and fresh-process restoration through the existing evaluator codec. No replacement AD or serialization system is justified.

## 5. Scientific tests, multiloop examples and completion

### Core correctness matrix

Validate:

- Analytic pole controls and above-threshold bubbles, including nearest-zero counterexamples.
- HEPKit/OneLOop triangles and boxes below, across and above thresholds.
- Taylor/IBP, repeated propagators, complex numerators, higher endpoint derivatives and complete Laurent cancellations.
- Fixed-strength invariance and agreement between fixed and dynamic results.
- Causal branches at stationary points, subtraction faces and exact contributions.
- Excessive strengths, intermediate-homotopy violations, \(U\)-branch violations, degenerate gradients and unresolved arithmetic.
- None/Korobov2/Korobov3; QMC and Havana/discrete MC where supported; ordinary and serial execution.
- Eager/SymJIT/precision-rescue agreement, artifact reload and checkpoint resume.
- Independent derivative checks and symbolic verification of specialized root formulas.
- `always`, `pilot` and `off` agreement using identical production points, unchanged RNG streams, correct validation reporting and measured overhead removal.

### Physical \(gg\to hh\)

Extend the existing full one-loop incoming-\(++\) example to an above-top-threshold point, initially \(\sqrt{s}=400\) GeV. Compare against HEPKit and MadLoop, including pole cancellations and Ward checks. Follow with the physical `gghh_double_box` example.

### LTD-paper multiloop suite

Add reproducible examples from the paper:

| Tier | Examples |
|---|---|
| Required two-loop | Table 6 four-point `2L4P.b`, kinematics `K1` and massive `K1*`; Table 2 six-point `2L6P.a.I`. |
| Required three-loop | Table 8 physical ladder `3L4P.K1`, with its analytic reference. |
| Extended stress test | Table 3 four-loop fishnet `4L4P.a.I`. |

Use the paper’s ancillary graph and kinematic definitions, checking topology identity because its prose contains a `2L4P.b/c` naming inconsistency. The five-/six-loop examples without thresholds do not count as contour tests. [Paper, §7.1 and Tables 2–8](https://arxiv.org/html/1912.09291v2)

Pin the [v2 source archive](https://arxiv.org/src/1912.09291v2) and its digest. Import the selected ancillary records through HEPKit primitives into native reproducible fixtures, preserving momentum routing, masses, propagator powers, normalization, reference precision and uncertainties.

Run analytic and two-loop gates before the three-loop case. Keep the four-loop case as a bounded extended test rather than an initial delivery blocker.

### Measurements and acceptance

Measure generation time, evaluator size, aggregate RSS, average and maximum sample time per sector, root overhead, validation overhead and time to \(10^{-3}\) relative uncertainty in the last requested complex Laurent coefficient on eight cores.

Compare fixed and dynamic modes using independent repeated runs, including checked-pilot/unchecked-production timing. Use pole clearance as a diagnostic alongside actual convergence.

Reuse published analytic and numerical references with their stated uncertainties. Published LTD timings provide context, not directly comparable FastSecDec speed claims. Run fresh pySecDec comparisons only within the earlier ten-minute/15-GB reference budget.

The completed dynamic mode must implement the smooth causal bounds, support optional runtime validation, and demonstrate correctness and practical behavior on physical one-, two- and three-loop tests. Finish only after interfaces, scientific checks, performance review and ecosystem audits pass.

---

## Subsequent user requirement — variance monitoring (2026-10-09)

> Continue as planned, but during testing also monitor how much of a better variance you can achieve by using dynamical vs static lambda (once both are implemented).

Once both modes are implemented, record fixed-versus-dynamic variance ratios at
equal production work, per sector and for the last requested complete complex
Laurent coefficient. Preserve real–imaginary covariance and distinguish
pointwise integrand variance from the variance between independent shifted-QMC
replicas; the latter determines the QMC error estimate. Use repeated independent
runs, matched sampling designs and validation policies, and record all strength,
cap and safety-fraction settings. Any tuning uses separate pilots and is frozen
before the comparison's production samples.

Report sampling cost, variance reduction per unit work and time to the requested
accuracy together. Record neutral or worse outcomes as well as improvements;
larger displacement alone is not evidence of reduced variance or faster
convergence. This monitoring extends the performance acceptance work above and
does not delay the fixed-mode correctness milestone until dynamic mode exists.

## Subsequent user requirement — upstream fixes (2026-10-09)

> Continue as planned, but when you find such issue make a PR to the corresponding crate for a fix with BenRuijl as a reviewer. Make this PR owned by ValentinHirschi (valentin.hirschi@gmail.com)

Publish validated narrow dependency corrections as upstream pull requests from
the authenticated `ValentinHirschi` account, with commits authored by
`ValentinHirschi <valentin.hirschi@gmail.com>`. Request `benruijl` as reviewer and
record any upstream permission restriction rather than claiming the request
succeeded. Attach every created pull request to the task. Keep independent
owner changes separately reviewable; retain executable regressions and source
reuse evidence. This authorizes publishing the previously prepared owner fixes.

## Subsequent user priority and stopping checkpoint (2026-10-10)

The following instruction changes the immediate delivery priority and adds an
explicit pause after that delivery. It does not retroactively turn unfinished
multiloop accuracy gates into successful results.

### User instruction, verbatim

Continue as planned, but I want you now to not insist much further on validation on the contour deformation complicated cases (keep the current passing ones as is, but if the issue is only too-large variance then don't insist), and instead focus on applying this new contour deformation strategies to the physical double-box example of gg_hh, but this time for sqrt(s) above threshold (say at 1000 GeV).

We don't have a benchmark there yet, but I would like you to optimize the generation and runtime of both deformation strategy as much as you can and report the resulting generation and runtimes, as well as resulting best-found variance for the final total result in both cases (after 5 minute runs on 50 cores), with both `qmc` and `discrete_mc` strategies and for both the `fixed` and `dynamical` deformation.
Once you got those numbers and have picked up all low-hanging fruits optimization of both generation and runtime, clean-up and push a version with green-local gate, return these numbers in a table for me, and pause the goal and stop yourself so we can decide together how to proceed.

### Immediate delivery goal

Deliver the native incoming-`++` physical D05 double-box example at
`sqrt(s)=1000 GeV`, retaining its established model, masses, scattering angle,
numerator, colour contraction and normalization. Improve generation and
sampling cost where measured evidence identifies straightforward changes, and
report fixed/dynamic results for both QMC and discrete MC after five-minute
integration runs with 50 workers. Preserve the full complex Laurent vector and
covariance; report the final finite coefficient, its covariance trace, joint
sampling uncertainty, actual work, generation time, and setup/sampling costs.
There is no independent numerical benchmark for this point, so do not imply
validated physical accuracy from agreement of noisy runs alone.

Keep existing passing complex-case evidence. Do not continue costly LTD
refinement solely to reduce its variance. Preserve the interrupted K1 accuracy
checkpoint and honest unfinished results; no further eight-dimensional physical
probe or three-loop campaign is required for this stopping checkpoint.

Use the existing ordinary caller-owned 50-thread pool for both integration
methods when residency permits; discrete MC cannot use serial sector loading.
Do not equate independent CLI replicas with a single 50-worker integration.
Select useful fixed strengths and dynamic construction/caps using bounded
preliminary checks, freeze the final settings, and identify the selected
construction explicitly. Keep preliminary and final sampling identities
separate. Measure the five-minute integration budget from the first native
integration-status event, including worker/context setup and Havana adaptation;
report artifact restoration and causal pilot setup separately. Existing status
events can lag the first worker dispatch while other contexts are prepared, so
do not claim an exact first-sample clock. Retain only statistically valid
accepted work at interruption and report any finishing overshoot or unfinished
allocations.

Delegate input/generation, integration/performance, and independent scientific,
seed, timing and reuse reviews. Finish appropriate local scientific tests,
formatting and lint checks for the delivered changes. Commit and push the
validated result on `contour_deformation`, return the four-way comparison table,
then **set the active goal to paused and stop** under this explicit user
authorization. Do not mark the broader Phase B goal complete.

## Subsequent report and resource requirements (2026-10-10)

### User instruction, verbatim

Continue as planned, but also produce a typst file in the docs (with the compiled PDF also pushed) which describes the two deformation styles and corresponding formulaes in first section (about 3 pages max), then in a second section the results when applied to g g > h h with fastSecDec (one, page, reporting the generation, runtime, and convergence metrics), and a third section detailing some of the implementation details especially regarding performance.
Don't get to hang-up on the 8 GiB caps, and lift it to 100 GB, but really still make sure to consider low-hanging optimizations for keeping runtime and memory under control. And also consider adding an option for computing the jacobian determinant for the deformation using dualized evaluation only (this should significantly help generation, and you can report on the correspoding runtime performance then).

### Delivery additions

Publish a self-contained Typst report and its compiled PDF under `docs/`.
Section one explains fixed and dynamic deformation, the smooth causal bounds,
full Jacobian and branch/subtraction conventions in at most approximately three
pages. Section two occupies one page and reports the native 1000 GeV double-box
input, generation, runtime and convergence evidence for the four requested
five-minute, 50-core runs. Section three explains the performance-relevant
implementation decisions and measured tradeoffs. Keep unmeasured entries
explicitly pending until the campaign completes; distinguish joint estimator
variance from pointwise variance and preserve the full complex covariance.

Raise the campaign aggregate process-RSS limit to **100 GB
(100,000,000,000 bytes)**. Keep measured memory reporting and straightforward
optimizations; do not spend effort forcing this workload below the superseded
8 GiB cap. Preserve prior capped attempts as failed admission measurements.

Investigate a selectable Jacobian evaluator using existing native dual and
matrix facilities instead of materializing the symbolic determinant. Verify
complete higher subtraction derivatives, dynamic-strength derivatives,
factorwise branches, saved-program restoration and precision rescue. Compare
generation size/time and sampling cost against the shared symbolic determinant
before choosing a campaign implementation. Document native reuse evidence and
any unresolved limitation; never substitute a first-derivative-only Jacobian
for the full jets required by endpoint subtraction.

## Serial residency and concurrent work clarification (2026-10-10)

### User instructions, verbatim

You said earlier:
"""
All 30 dynamic sectors have completed mapping and are now compiling. This step uses substantially more memory than fixed mode, so I’m keeping it at four workers and checking the dual-Jacobian option as a possible improvement. A separate short MC check will verify memory use with 50 workers before the timed runs.
"""
Remember that the user of the `--serial` option is precisely to alleviate these issues.

(also ignore in the future impact of concurrent activity on timings for this machine, and by that I mean don't let that prevent you from running things in parllel).

### Execution clarification

Use the existing serial generation path for bounded residency. Verify actual
coordinator and child-process RSS, disk-backed preparation records, and process
exit before reusing a worker slot. Distinguish the size of an active sector
from retention of completed sectors; lowering worker count must not conceal
unbounded retention. Global preparation barriers for exact symmetry and formula
reuse do not authorize keeping all mapped expressions in memory.

Run independent generation, builds, checks and integration measurements in
parallel when resources permit. Do not wait for a quiet machine or use
concurrent activity as a reason to postpone measurements. Preserve each
integration's 50-core allocation, deterministic sampling identities, memory
accounting and separate results. Select each prescription using preliminary
work before its final run; final results must not feed subsequent tuning.
