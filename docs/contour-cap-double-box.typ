== Physical double-box: increasing the saved cap
The physical D05 double-box at $sqrt(s)=1000$ GeV is reused without symbolic
generation or optimization. Its historical programs have *Horner zero*:
fixed uses a symbolic deformation Jacobian, polynomial dynamic uses the
contour-only dual Jacobian, and both retain symbolic endpoint IBP. This is a
runtime cap study of those existing programs, not a newly generated Horner-ten
comparison. All quantities below refer to this one diagram with the earlier
normalization, not the complete $g g -> h h$ amplitude.

The earlier 50-core campaign used $L=10^(-6),S=.8,R=1$, hence
$lambda(x)<=8 times 10^(-7)$. A scout now scans
$L=10^(-6),10^(-5),10^(-4),10^(-3),.01,.1,1,10,100$ at $S=.8,R=1$.
It retains numerical sectors `[0,3,6,11,26,28,29]`, with exact offsets
explicitly excluded, and 1024 points × eight shifts per sector. These are
partial-integral estimates on one paired tuning seed.

#table(columns: (1.45fr, .7fr, 1.4fr), inset: 4pt, stroke: .35pt + rgb("cbd5e1"),
  table.header([*Scout prescription*], [*$V$*], [*Interpretation*]),
  [Fixed $lambda=10^(-6)$], [1049.39], [Historical fixed control],
  [Dynamic $L=10^(-6),S=.8$], [2529.35], [Historical dynamic control],
  [Dynamic $L=10^(-5),S=.8$], [1376.14], [Best cap in the first scan],
  [Dynamic $L>=.01,S=.8$], [337343], [Saturation through cap 100],
  [Dynamic $L=10^(-5),S=.4$], [213.565], [Selected after the R/S scan],
)
Here $V$ is the trace of the finite complex estimator covariance. A subsequent
nine-arm scan varies $S=.4,.8,.95$ and $R=.01,.1,1,10$ in the predeclared
combinations retained in the evidence. The selected $L=10^(-5),S=.4,R=1$
minimizes both $V$ and its product with measured evaluation time. The two
repeated controls reproduce their original complete vectors and covariance
exactly. Poor admitted results remain visible: $L=1,S=.8,R=.01$ gives
$V=5.484 times 10^8$ on this same partial scope.

=== Why a much larger cap is not automatically better
All 198 largest-weight cross-cap replays pass, including 63 original-vector
checks. At a recorded sector-29 point, raising the cap to 100 yields only
$lambda(x)=2.528574 times 10^(-4)$. The causal, displacement, positive-factor
and cap shares of $H$ are approximately .57728, .42252, .00021 and
$10^(-11)$. Increasing the cap cannot remove these other sufficient bounds.
At that point the full determinant magnitude is 281409, versus 786384 with
the strength derivative artificially frozen for diagnosis. Thus the observed
large weight does not justify omitting the dynamic derivative; it already
reduces the determinant. Base-chart attribution and the complete residual's
other subtraction-face callbacks are recorded separately.

The useful setting therefore increases $L$ tenfold while reducing $S$.
This permits larger local strengths than the original small cap while
avoiding some of the amplification from the larger safety fraction. It is
a measured choice for this diagram, not a universal deformation default.

#pagebreak()
== Full double-box confirmation on fresh samples
The three prescriptions were frozen before seeds `202610104201` and
`202610104202`. Each run covers all 30 six-dimensional residual kernels,
4096 points × eight complete Kuo33002 shifts per sector, Korobov3, and the
native exact offset once. Same-seed point/weight hashes match across all
prescriptions; the two seeds have different streams. Native complete Laurent
vectors and their covariance are retained, including shared-shift sector
cancellations. Each result is separate: no pooling across seeds or settings.

#table(columns: (1.45fr, 1fr, .7fr, .9fr), inset: 5pt, stroke: .35pt + rgb("cbd5e1"),
  table.header([*Prescription*], [*$V$: seed 1 / 2*], [*Sample s*], [*Eval. µs avg/max*]),
  [Fixed $lambda=10^(-6)$], [1817.77 / 1511.09], [7.510], [6.769 / 7.695],
  [Old dynamic $L=10^(-6),S=.8$], [3517.00 / 2575.89], [27.239], [26.690 / 44.160],
  [Selected $L=10^(-5),S=.4$], [521.39 / 335.11], [27.194], [26.692 / 42.222],
)
Both dynamic rows have $R=1$. The table averages sampling times and sector
means over the two seeds; its maximum is the largest sector mean in either
run. Sampling excludes loading, pilot, coordinate hashing and largest-point
bookkeeping, but includes the integration coordinator. Evaluator timings
include precision rescues. These are one-core-per-run matched-work QMC
measurements, not replacements for the earlier 50-core, 300-second campaign.

Relative to fixed strength, selected dynamic reduces variance by
*3.486× and 4.509×*. Relative to the old dynamic prescription, the reductions
are *6.745× and 7.687×*. Mean separately measured variances are 1664.43 fixed,
3046.45 old dynamic and 428.25 selected dynamic. These descriptive averages
are not a combined integration estimate or a statistical confidence interval.

The corresponding fixed-to-selected $V t$ ratios are *.964 and 1.244*;
there is no consistent timing advantage across both runs. The mean products
are 12500.88 fixed, 82916.75 old dynamic and 11645.59 selected dynamic.
Thus tuning removes much of the earlier dynamic variance penalty, while
the contour evaluation cost remains material. Two short confirmations do
not establish an asymptotic convergence law or an optimum.

#table(columns: (.55fr, 1.5fr, 1.5fr, 1.5fr), inset: 5pt, stroke: .35pt + rgb("cbd5e1"),
  table.header([*Seed*], [*Fixed finite mean*], [*Old dynamic finite mean*], [*Selected finite mean*]),
  [4201], [$26.17+36.63i$], [$17.85+18.88i$], [$31.50+42.33i$],
  [4202], [$52.47+28.40i$], [$42.52+19.65i$], [$73.43+46.12i$],
)
These short-run estimates remain noisy: their joint standard errors are
$sqrt(V)$ from the first table. They do *not* claim per-mille accuracy, and
there is still no independent physical D05 reference at this point. The
purpose is matched-work variance and cost comparison, with the earlier
longer estimates retained independently.

All six arms pass Pilot16, with zero optional production causal checks and
precision rescue enabled. Restore plus integrity verification takes
7.04–7.07 s fixed and 10.00–10.17 s dynamic; binding takes .006 and .13 s.
Pilots take 14.27–14.31 s fixed and 9.19–9.23 s dynamic. Maximum sampled
process-group RSS is 257.38 MB fixed and 636.97 MB dynamic, including
setup. At most two independent invocations run concurrently; the complete
batch closes in 122.58 s with 1.238 GB aggregate peak RSS and no limit hit.
No new generation, MC sweep, or sign-aware double-box generation is claimed.

Full native estimates, settings, timings, covariance and work-identity
evidence are retained in `contour-cap-double-box-results.json`. The
independent scientific and ecosystem review is
`reviews/contour-cap-ansatz-study.md`. Existing parameters suffice for this
measured improvement; a new Jacobian-distortion constraint remains a proposal.
