== Scalar topology comparison: generation and sampling cost
The four native scalar inputs use numerator one and $D=4-2epsilon$, through the finite term. All six constructions retain symbolic endpoint IBP, SymJIT O2, Horner zero, common-pair cap 1000 and initial native series width two. Generation uses one in-process caller and one compiler core; sampling uses one core per task, with at most two tasks running on distinct physical cores. These are separate from the 50-core double-box measurements.

#set text(size: 8.3pt)
#table(columns: (1.1fr, .92fr, .32fr, .62fr, .72fr, .93fr, 1.06fr, .74fr), inset: 3pt, stroke: 0.35pt + rgb("cbd5e1"),
  table.header([*Case*], [*Contour*], [*J*], [*Gen. s*], [*Gen. MB*], [*IR kB*], [*Eval. µs avg/max*], [*Run MB*]),
  [Triangle], [Fixed], [S], [0.022], [381.26], [2.202/3.216], [0.248/0.281], [16.13],
  [], [Fixed], [D], [0.022], [450.16], [1.710/3.402], [0.241/0.271], [16.05],
  [], [Polynomial], [S], [0.027], [435.97], [5.616/7.442], [0.503/0.582], [16.33],
  [], [Polynomial], [D], [0.028], [454.82], [4.470/7.785], [0.483/0.549], [16.44],
  [], [Sign-aware], [S], [0.025], [458.05], [5.616/7.442], [0.503/0.581], [16.45],
  [], [Sign-aware], [D], [0.028], [406.97], [4.470/7.785], [0.480/0.543], [16.24],
  [Box], [Fixed], [S], [0.032], [471.89], [3.212/5.166], [0.222/0.228], [16.10],
  [], [Fixed], [D], [0.031], [476.60], [2.492/5.035], [0.218/0.222], [16.02],
  [], [Polynomial], [S], [0.039], [469.16], [8.366/11.91], [0.469/0.480], [16.62],
  [], [Polynomial], [D], [0.043], [467.39], [6.626/12.97], [0.460/0.471], [16.39],
  [], [Sign-aware], [S], [0.038], [466.14], [8.366/11.91], [0.469/0.481], [16.39],
  [], [Sign-aware], [D], [0.043], [467.23], [6.626/12.97], [0.461/0.479], [16.66],
  [Sunrise], [Fixed], [S], [0.020], [59.79†], [0.846/0.975], [0.160/0.171], [15.80],
  [], [Fixed], [D], [0.019], [59.78†], [0.812/1.065], [0.158/0.170], [15.84],
  [], [Polynomial], [S], [0.021], [59.80†], [0.862/0.975], [0.160/0.171], [15.90],
  [], [Polynomial], [D], [0.021], [59.79†], [0.828/1.065], [0.159/0.171], [16.01],
  [], [Sign-aware], [S], [0.021], [59.79†], [0.862/0.975], [0.159/0.171], [15.88],
  [], [Sign-aware], [D], [0.021], [59.82†], [0.828/1.065], [0.159/0.171], [16.05],
  [Kite], [Fixed], [S], [0.358], [21.48], [40.56/85.37], [0.328/0.351], [21.18],
  [], [Fixed], [D], [0.269], [18.88], [44.62/100.26], [0.329/0.369], [20.33],
  [], [Polynomial], [S], [1.967], [62.48], [674.99/1082], [3.051/4.733], [58.08],
  [], [Polynomial], [D], [3.801], [59.31], [692.68/1240], [3.134/4.946], [57.33],
  [], [Sign-aware], [S], [4.022], [114.91], [1237/2016], [10.20/19.15], [92.34],
  [], [Sign-aware], [D], [12.30], [105.32], [1156/1877], [8.558/14.94], [81.16],
)
#set text(size: 10.2pt)
S/D denotes symbolic/contour-only dual Jacobian. Gen. time includes native generation and evaluator compilation, excluding input/reference evaluation and serialization. IR gives exact / lowered JIT program sizes. Generation RSS covers the whole input/reference, generation, compilation and saving process; it therefore has a broader scope than the generation timer. Memory is sampled owned-process peak RSS in decimal MB; run peaks include loading, pilot and all three sampling epochs. Evaluator costs include precision rescues: average is the mean of sector means over the two final-epoch runs, maximum is the largest sector mean in either run. They exclude caller overhead and are not maximum single-point latencies. A dash denotes missing accepted evidence.
† The original sunrise processes finished between RSS polls. These six values are kernel high-water measurements from separate identical generation-only repeats, retaining the original generation timings. GNU time’s parent RSS was independently below 1.81 MB, excluding the inherited launcher-memory floor. All other generation RSS values are periodic samples; short peaks may be missed.

The independent references are native OneLOop expressions for the massive triangle and box, a Symbolica Laurent expansion of the gamma-function identity for the massless sunrise, and the published convergent small-momentum series for the kite. Points are $C_0(0,0,5;1,1,1)$, $D_0(0,0,0,0,5,-1;1,1,1,1)$, $p^2=1$ for the sunrise, and $p^2=3/1000$, $m^2=1$ for the three-massive/two-massless kite. The sunrise probes causal branches and UV endpoint subtraction; it has no interior causal zero.
References: #link("https://arxiv.org/abs/1007.4716")[OneLOop]; #link("https://arxiv.org/abs/hep-ph/9605392")[Fleischer, Smirnov and Tarasov, zero-threshold expansion]. Native normalization and routing checks accompany the fixtures.

#pagebreak()
== Scalar convergence at matched work and elapsed time
Each curve uses two run seeds and separate complete lattice epochs of 1024, 2048 and 4096 points, with eight shifts, Kuo33002 and Korobov3. The displayed line is the arithmetic mean of the two native joint standard errors; shading spans the two observations. No estimates or covariance matrices are pooled across lattice sizes, seeds or constructions. The band is not a confidence interval.
#image("contour-scalar-convergence-work.svg", width: 100%)
#image("contour-scalar-convergence-time.svg", width: 100%)
Sampling time excludes separately measured coordinate hashing and includes native worker-context creation, precision-cache warm-up and caller work. Each x-coordinate is the mean of the two epoch times. These short runs illustrate finite-work behavior; they do not establish an asymptotic convergence rate.
#pagebreak()
== Scalar findings and limits of the comparison
All 24 generated owners and all 144 lattice epochs completed. All 72 symbolic/dual Jacobian pairs have identical actual coordinate and weight hashes, with full-vector means agreeing to $1.47 times 10^(-16)$ on the scale $max(1, abs(I))$. Every case admits the common fixed strength or dynamic cap $0.1$; dynamic modes use $S=.8$, $R=1$. This is a common-cap comparison, not an optimization of each prescription’s variance. In particular, dynamic strength is at most $.08$, whereas fixed strength is $.1$.

#set text(size: 9pt)
#table(columns: (1fr, .9fr, 1.05fr, .7fr, .8fr, .75fr, .8fr), inset: 4pt, stroke: 0.35pt + rgb("cbd5e1"),
  table.header([*Case*], [*Contour*], [*Mean $V$*], [*$V/V_F$*], [*Rel. SE %*], [*Slope $p$*], [*Time ms*]),
  [Triangle], [Fixed], [5.020e-16], [1.00], [2.07e-06], [8.39], [33.87],
  [], [Polynomial], [7.888e-14], [157.13], [2.6e-05], [7.13], [51.06],
  [], [Sign-aware], [7.888e-14], [157.13], [2.6e-05], [7.13], [50.98],
  [Box], [Fixed], [6.354e-06], [1.00], [0.636], [0.90], [34.14],
  [], [Polynomial], [3.963e-05], [6.24], [1.59], [0.71], [50.77],
  [], [Sign-aware], [3.963e-05], [6.24], [1.59], [0.71], [50.69],
  [Sunrise], [Fixed], [1.260e-21], [1.00], [1.72e-09], [4.39], [15.36],
  [], [Polynomial], [1.260e-21], [1.00], [1.72e-09], [4.39], [15.31],
  [], [Sign-aware], [1.260e-21], [1.00], [1.72e-09], [4.39], [15.32],
  [Kite], [Fixed], [4.579e-04], [1.00], [0.457], [0.96], [233.31],
  [], [Polynomial], [9.969e-04], [2.18], [0.676], [0.82], [1227.83],
  [], [Sign-aware], [9.969e-04], [2.18], [0.676], [0.82], [3815.49],
)
#set text(size: 10.2pt)
The table uses symbolic J, with each entry a descriptive mean over the two runs at 4096 points × eight shifts per sector. $V$ is native estimator variance, $V_F$ the fixed result, and $p=log_4("SE"_(1024)/"SE"_(4096))$ uses the two-seed mean standard errors. The large triangle slope is an observed short-range decrease, not a demonstrated scaling law. No new combined estimate is constructed.

*Fixed strength wins at these settings.* The polynomial dynamic variance is about 157 times larger for the triangle, 6.24 for the box and 2.18 for the kite at matched work. The dynamic evaluator also costs more per point. These results do not rule out better choices of the caps or a benefit on different physical inputs.

*Two degeneracies explain overlapping curves.* The triangle and box have quadratic residual $F$ and linear $U$. There are no higher odd causal terms or even positive-factor terms to distinguish the two dynamic envelopes, so polynomial and sign-aware reduce to the same radius equation. For the massless sunrise, monomial extraction leaves constant residual $F$: all dual build records report zero surviving contour derivative slots, and all six estimates agree. It is a useful branch/subtraction control, not evidence that six active deformation algorithms have identical cost.

*The kite distinguishes the full envelopes.* Its sign-aware construction offers essentially the same variance as the polynomial bound at this admitted cap, at greater generation and sampling cost. Contour-only dual J reduces the sign-aware mean evaluator cost from 10.20 to 8.56 µs per sector-point, but generation rises from 4.02 to 12.30 s. It is therefore not a universal generation optimization. The complete timing and memory records include both constructions.

At the final lattice size, the largest finite-reference error norm divided by the native joint standard error is 1.88 across the suite. This descriptive check is consistent with the references; two run seeds cannot calibrate uncertainty coverage. There are no failed, unstable or cutoff-zero evaluations, no arbitrary-precision rescues, and no optional production causal checks. Double-double precision rescues remain enabled and are included in the measured cost. Full Laurent vectors, including the sunrise pole, are retained in the JSON.

The campaign took 90.23 s with 0.961 GB aggregate sampled RSS, including its coordinator and at most two active processes. The optimized example link took 143.04 s and the OneLOop reference leaf 4.83 s; neither is part of generation timing. The generation/reference/protocol controls and independent ecosystem review pass. The wider Phase B scientific acceptance programme remains unfinished.
