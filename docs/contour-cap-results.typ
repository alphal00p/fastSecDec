== Cap tuning with the normal Horner setting
The follow-up uses the same connected scalar kite, now generated with the
public *Horner-ten* default. All three rows retain symbolic endpoint IBP and
a symbolic deformation Jacobian. Its 22 source charts give 11 numerical kernels.
Changing $L$, $R$ or $S$ reuses these saved evaluators without regeneration or
Horner optimization. SymJIT O2 and the earlier physical normalization remain
unchanged.

#set text(size: 9pt)
#table(columns: (1.15fr, .7fr, .75fr, .95fr, 1fr), inset: 4pt, stroke: .35pt + rgb("cbd5e1"),
  table.header([*Construction*], [*Gen. s*], [*Peak MB*], [*Exact / JIT kB*], [*Eval. µs avg/max*]),
  [Fixed], [.455], [56.62], [24.16 / 57.35], [.274 / .297],
  [Polynomial], [2.525], [68.31], [190.59 / 409.78], [1.742 / 2.404],
  [Sign-aware], [5.039], [115.95], [236.90 / 473.36], [2.566 / 3.588],
)
#set text(size: 10.2pt)
Generation is native symbolic construction plus evaluator compilation. Peak
memory is GNU time's process high-water mark over input/reference, generation,
compilation and saving. Sampling costs use the selected settings below,
including precision rescues. The average is the mean of sector means over two
confirmation runs; the maximum is the largest sector mean in either run.
Horner ten reduces the polynomial/sign-aware lowered IR from the historical
1.08/2.02 MB to .41/.47 MB. Those older programs remain labelled Horner zero.

A predeclared tuning scan used $L=0.03,0.1,0.3,1,3,10$, with $S=.8,R=1$,
and a separately tuned fixed strength. Fixed $.5,.6,.8,1$ failed the native
causal pilot; their failures are retained rather than silently shrinking them.
A second scan varied $R=.25,4$ and $S=.5,.95$. Choices were frozen before two
fresh confirmation seeds. Each arm uses 4096 points × eight shifts per sector,
Kuo33002 and Korobov3. Actual point/weight hashes match across prescriptions
within each seed, and differ between seeds. All estimates retain native complex
covariance and exact offsets; separate arms and seeds are never pooled.

#image("contour-cap-kite.svg", width: 100%)
The left panel is one tuning seed. The right shows the two fresh variance
observations and their descriptive mean, not a confidence interval.

#set text(size: 9pt)
#table(columns: (1.2fr, 1.3fr, .8fr, .75fr, .9fr), inset: 4pt, stroke: .35pt + rgb("cbd5e1"),
  table.header([*Construction*], [*Settings*], [*Mean $V$*], [*Sample s*], [*Mean $V t$*]),
  [Fixed], [$lambda=.1$], [7.305e-4], [.210], [1.537e-4],
  [Fixed, selected], [$lambda=.4$], [1.683e-5], [.217], [3.640e-6],
  [Polynomial], [$L=.1,R=1$], [1.439e-3], [.726], [1.045e-3],
  [Polynomial], [$L=1,R=1$], [1.015e-5], [.781], [7.944e-6],
  [Polynomial], [$L=10,R=1$], [1.494e-3], [.817], [1.220e-3],
  [Polynomial, selected], [$L=1,R=.25$], [5.072e-6], [.751], [3.802e-6],
  [Sign-aware, selected], [$L=1,R=.25$], [5.072e-6], [1.049], [5.314e-6],
)
#set text(size: 10.2pt)
All dynamic rows use $S=.8$. $V$ is the trace of the finite estimator covariance.
Times exclude loading/pilot and the measured coordinate-hashing/top-point
retention spans. $V t$ is averaged per run, not formed from a pooled estimate.
It is a short-run efficiency diagnostic, not an asymptotic convergence law.

#pagebreak()
== What the largest-weight points show
The selected dynamic choices reduce confirmed variance by *3.318×* relative
to tuned fixed $lambda=.4$. Their individual variances are
$[4.134,6.010] times 10^(-6)$, versus fixed
$[1.633,1.733] times 10^(-5)$. The polynomial runtime cost offsets this gain:
its mean $V t$ is 1.044 times fixed, while sign-aware is 1.460 times fixed.
There is *no demonstrated sampling-speed advantage* in this small comparison.
Including diagnostic instrumentation in the timer would misleadingly favor
the slower method, because the same bookkeeping is a larger fraction of the
fixed evaluator's cost. Separately measured setup and sampled RSS remain in
the native evidence.

The large-cap deterioration is reproducible on the fresh seeds. To investigate
it, the union of recorded largest-weight points was replayed under every cap,
separately from production timing. There are 282 replayed points per dynamic
construction and 66 same-arm full-vector parity checks each. No samples or new
estimates enter this diagnostic. Symbolica supplies derivatives/evaluators;
the native radius solver and matrix determinant supply the remaining analysis.
Diagnostic strengths agree with the actual callback strengths to within
$1.8 times 10^(-15)$.

For example, at one identical recorded point in residual kernel 10
(source chart 15, shift 7, point 431), polynomial deformation gives:

#table(columns: (.5fr, .75fr, .75fr, .8fr, 1.1fr), inset: 5pt, stroke: .35pt + rgb("cbd5e1"),
  table.header([*$L$*], [*$lambda(x)$*], [*$abs(F(z))$*], [*$abs(det J)$*], [*Weighted real output*]),
  [.1], [.0800], [1.087], [1.040], [.1722],
  [1], [.7989], [1.088], [5.322], [−.7322],
  [10], [6.728], [1.201], [820.8], [−108.94],
)
The magnitude $abs(F(z))$ does not approach zero at this point as the cap increases;
the map strongly amplifies the density. The full Jacobian is smaller than
the frozen-strength diagnostic determinant at $L=10$ (821 versus 1194).
Thus this evidence does not support blaming only the rank-one
$v (nabla lambda)^T$ term, and does not justify dropping it. Other recorded
maxima in kernels 5, 7 and 9 remain over 99.2% cap-dominated at $L=10$,
with determinant amplification as the cap grows. These observations explain
specific large weights; they are not a proof that one point determines the
whole covariance.

At the chart center the displacement constraint can already dominate, so a
center-only probe would miss this behavior. The four nonnegative shares of
$H$ and the actual weighted points distinguish these regimes. Reducing $R$
from 1 to .25 at $L=1$ improves the confirmed polynomial variance by about
2×; increasing it to 4 worsened the tuning result. Increasing $S$ from .8 to
.95 also worsened that result. Causal freedom therefore needs variance tuning.
The existing runtime parameters provide a measurable improvement without a
new production ansatz.

A possible later extension is a nonnegative smooth envelope term
$theta r^2 norm(D v)_F^2$, with $theta>=0$. It preserves the existing sufficient
causal bound and would constrain $lambda norm(D v)_F$. It does not bound
$v (nabla lambda)^T$ and has not been implemented or benchmarked. Any such
extension must retain all symbolic derivatives, native checker consistency,
and artifact identity; the present results make no claim for its performance.

=== Reproducible small source selections
The maintained kite TOML card accepts `generation.source_sectors = [2, 5]`.
These are original geometry-chart IDs before symmetry and subtraction; this
pair produces one symmetry-weighted numerical kernel. Native metadata saves
the original total and local-to-original mapping. Inspection and numerical
results label the selected contribution, including after portable/indexed
reload. Omit the option for the full integral. This permits one- or two-sector
generation experiments without confusing them with complete amplitudes.

Full native estimates, covariances, stream hashes, timings, generation sizes,
pilot failures and settings are retained in `contour-cap-scalar-results.json`;
the independent review is `reviews/contour-cap-ansatz-study.md`.
