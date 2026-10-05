# Next independent references: rank-two triple box and hard four-loop density

Source-only scheduling proposal after the accepted off-shell scalar reference.
No new generation, package build or sampling is authorized by this document.
The completed scalar fixture and earlier failures remain unchanged. The next
scientific gates are the off-shell rank-two numerator and the explicit hard
four-loop positive-orthant density, each with its complete physical Laurent
vector through the highest requested order 0.

## What the completed scalar run establishes

Scalar generation took 136.321 seconds and produced 1182 ordinary sectors. Its
original 600-second whole-process deadline expired during serial compilation.
A separately reviewed copied-package continuation finished native `make -j8`
in 324.766 seconds; that started from 136 completed FORM stamps and 412 objects,
so it is not the cost of a clean eight-worker build. Its 180-second numerical
bound then expired after three coefficient calls. The successful fresh numeric
attempt spent about 125 seconds before the first QMC call and completed all four
calls in 233.887 seconds of numerical-process wall time.

These are diagnostic observations with permitted other-CPU activity. They show
why a shared 600-second budget or 180-second ordinary numerical cap is poorly
matched to these phases. They neither predict the two remaining cases' costs
nor establish a parallel speedup. Ordinary `pylink_integral.hpp` explicitly
ignores the amplitude `wall_clock_limit` argument: external watchdogs alone
bound wall time, while the existing QMC constructor's maxeval steering remains
effective. No proposed phase relies on an internal deadline.

## Proposed first bounds and scheduling

| Phase, per new case | Proposed external bound | CPU ownership | Outcome required before next phase |
| --- | ---: | --- | --- |
| Original native package generation | 600 s | one prescribed CPU; exclusive Symbolica slot | successful child exit/reap, generated metadata and input/source validation |
| Native ordinary FORM/C++ build | 1200 s | eight allowed distinct physical CPUs 0–7, native `make -j8`, FORMTHREADS=1/FORMOPT=2 | complete library and package/source hash record |
| Complete ordinary numerical call | 750 s | CPU 0, one numerical worker, fail-closed Symbolica guard | original complete tuple persisted before converter; no partial promotion |
| Guard controls and all phases together | 2600 s | phase limits clipped to remaining total | each unused stage explicitly not-started after any failure |

The 30-GiB process-tree RSS bound applies to every executed phase. Each output
directory is new and records all argv, tools, source hashes, stage exits and
watchdog outcomes. The generation child must have exited and been reaped before
another owner starts native Symbolica. Guarded make/numerical work can overlap
coordinated untimed native science on a disjoint CPU, with that overlap recorded.
It cannot overlap an exclusive performance measurement. No phase is silently
retried or extended, and no build resumes in a failed directory. A later
continuation, if justified, copies and verifies the full package bytes and
timestamps under a separately reviewed bound.

Rank-two generation is proposed first, after Pathfinder's existing-IBP
preparation and point-first oracle slot. Hard-four-loop generation follows only
an explicit runtime handoff; it does not start automatically while any other
Symbolica owner is active. After generation, inspect actual sectors, coefficient
range and generated source sizes before deciding whether the proposed 1200/750 s
phases remain a useful attempt. If native generation itself fails, preserve
that observation and reassess rather than spend the later phases.

## Frozen scientific callers and a separate phase wrapper

Keep both already reviewed sources byte-for-byte unchanged:

- `output/probes/run_projected_triple_together.py`, SHA256
  `c14bb4faef75a026d6d09ba83a8d836dd1aea3920e6140043552c6beabb63beb`.
  Its existing `--generate rank2 OUTPUT` and `--integrate rank2 OUTPUT` modes
  provide the native scientific operations without the old shared deadline
  orchestration. The latter retains its `wall_clock_limit=180` argument, documented
  as unused by the ordinary provider.
- `output/probes/run_hard_four_loop_together.py`, SHA256
  `f2d2a98b9f16d02c79c2dbe6400049145a8bebed22b73141c37a4be44293e83b`.
  Its existing `--generate OUTPUT` and `--integrate OUTPUT 180` modes likewise
  keep all scientific settings unchanged. Passing180 satisfies the frozen
  caller's argument admission; the separate 750-second external watchdog is
  authoritative, not that unused native amplitude argument.

A new ignored reference-only phase wrapper would own only preparation,
affinity, guards, native make, external bounds and recording. It would reuse
the reviewed helpers, record the unchanged public-mode commands above, and
repeat negative/import controls before unguarded generation and guarded work.
No caller source edit, monkeypatch, new algebra, estimator or driver is needed.
Concrete wrapper review and coordinator approval must precede execution.

## Rank-two scientific contract

Use the original literal routing and numerator
`k1(mu)*k3(mu)+2*k2(nu)*p1(nu)*k2(rho)*p2(rho)`, preserving coefficient-one
native repeated-denominator projection and active powers [1,1,2,2,1,1,1,1].
The measure remains three-loop `prod d^Dk/(i*pi^(D/2))`, D=4−2eps,
massless internal lines, all four external virtualities −1 and s12=s23=−2.
The original graph weight is 1. Native pySecDec Gaussian/tensor machinery owns
the numerator, raised-power measure, signs and Gamma factor. Do not copy the
scalar Gamma expression onto this input or add a manual factorial.

The ordinary package metadata and returned order range are observed facts,
not assumed equal to the scalar package. Preserve all physical coefficients
through 0, including any extra poles or measured zeros. Native retained original
and projected rank-two vectors both cover −3..0; compare the full key union
without inventing missing coefficients. Each native vector/covariance remains
unchanged and separate. Existing source/normalization checks, not agreement
alone, are prerequisites to accepting any reference.

Keep explicit seed 20261202, published cbcpt_dn1_100, Korobov3/no fit, N=8311/R=32,
maxeval=265952, epsrel=0.01/epsabs=1e−12 and one numerical CPU thread. If the native
logs confirm one allocation per coefficient, 8311×32 is its scalar
summed-coefficient point count. Total work, coefficient count, precision target
and uncertainties remain observed outcomes, not presumed from steering.

## Hard four-loop scientific contract

Use the already independently reviewed F-only native fan: nine original
ordered variables, complete positive orthant, density `U*F^(eps−3)`, unit
prefactor and measure `prod dx_i`. It is not a projective graph integral and
has no implicit Gamma multiplier. Native `other_polynomials=[U]` preserves
the transformed signed integer U factor and its extracted monomials at infinity;
native geometric decomposition and subtraction own that handling. Original
native/reference U and F hashes and whitespace-only equality checks remain
mandatory. No selected sector, changed integration support or projective
constraint is introduced.

Keep seed 20261219 and the same explicit lattice/transform/tolerance steering.
The retained native vector has orders −2..0 and 699 representative kernels, but
external sector count and coefficient range must be obtained independently.
Compare the complete physical key union after unit-normalization and full-domain
review; no similarity to a rounded historical target can substitute for it.
Ordinary `together=True` estimates each complete sector sum. Preserve the raw
tuple, every reported uncertainty and unavailable covariance rather than
recombining marginal sector errors. Neither fixed-work output nor the native
tolerance stop certifies the eventual highest-order 0.001 relative-error target.
