# Runtime integration settings and observations

`integrate BASENAME --integration-settings settings.toml` reads a TOML overlay.
Use either integration keys at the top level or a single `[integration]` table.
Only supplied keys override artifact defaults; explicit command-line flags apply
last. Nested tables merge, while arrays such as stability levels replace the
whole previous array. Unknown settings are rejected. Runtime parameters supplied
with `--parameters` and repeated `--parameter NAME=VALUE` remain evaluator inputs.

`--target-order 0` selects the complex ε⁰ coefficient for stopping. Its error is
`sqrt(C_RR + C_II)` from the full total covariance, and its mean magnitude is
`hypot(mean_R, mean_I)`. The accepted complete production allocation must satisfy
`error <= max(absolute_tolerance, relative_tolerance * magnitude)`. Other orders
and the full covariance are retained. Without an order selector the historical
all-component criterion applies. `--max-rounds N` bounds refinement; reaching
that bound is reported as a work limit, not as successful convergence.

## Stability policy

The default distance policy has three ordered numerical levels:

```toml
[stability]
mode = "distance"

[[stability.levels]]
precision = "f64"
minimum_effective_distance = 1e-3
escalate_for_large_weight_threshold = 0.9

[[stability.levels]]
precision = "double_float"
minimum_effective_distance = 1e-8

[[stability.levels]]
precision = "arbitrary"
minimum_effective_distance = 0.0
```

DoubleFloat uses native 106-bit arithmetic. Arbitrary precision uses 1000 decimal
digits (3322 bits). Higher precision uses the native eager evaluator; the local
f64 path retains SymJIT O2. Changing this runtime policy does not recompile the
symbolic expressions.

The effective distance uses the retained cancellation profiles:
`min_row product_i x_i^a_i`. Evaluation occurs after integration-coordinate
transformations and ignores coordinates without cancellation loss. Comparisons
use logarithms, so simultaneous small coordinates contribute to the same loss
budget without underflowing the distance itself.

The f64 and DoubleFloat levels may add a `power_thresholds` table whose keys are
exact original endpoint powers in `x^(-p + beta*eps)`. Missing keys inherit the level's global
threshold. For example, adding this immediately after the f64 level gives power
two a wider DoubleFloat band:

```toml
[stability.levels.power_thresholds]
"2" = 1e-2
```

Power-specific routing sums `a_i * log(1/x_i) / log(1/threshold(p_i))`
within each cancellation row. Reaching one advances past that numerical level.
The original powers come from exact generation metadata, not inferred Taylor
orders. Old artifacts with unresolved power associations require regeneration
before using these overrides. Global routing continues to work with old data.

The optional large-weight threshold compares each complex Laurent coefficient's
importance-weighted magnitude against a fraction of its previous sector maximum.
Any exceeded threshold advances the entire Laurent vector. The current candidate
does not set its own reference; a missing nonzero reference skips the check.
Only the final finite result updates the maximum. The highest numerical level
does not escalate large finite values into a cutoff zero. Checkpoint maxima
contain only admitted complete work; workers retain deterministic local state.

The distance policy is a heuristic. It removes the old zero-component underflow
heuristic that falsely escalated purely imaginary ggHH values. Nonfinite results
still advance through the stack, and persistent numerical failures remain errors.
To retain the earlier additional precision checks, select:

```toml
[stability]
mode = "validated"
```

That policy retains its variable arbitrary-precision checks (starting at the
stored precision policy, typically 128 bits). Its dashboard labels this class
`Arb (variable)` and reports the maximum successful-point precision, while
distance mode labels its fixed class `Arb<1000>`.

No point is deliberately zeroed by default. An explicit `unstable_cutoff` under
`[stability]` enables a smaller distance bracket that returns zero before any
evaluator call. Optional `unstable_power_thresholds` supplies exact-power
overrides. The settings and zeroed counts are saved. Sampling error does not
include the resulting cutoff bias.

## Live estimates and timing

`--status-interval-ms` controls dashboard, plain and JSON updates and defaults to
1000 ms. Cancellation polling is independent. Initial and terminal observations
appear immediately; table navigation redraws cached observations.

Use Up/Down, PageUp/PageDown, Home/End to select sectors, Left/Right to change
epsilon order, Tab or `s` to cycle sorting columns, Shift-Tab to cycle backwards,
and `r` to reverse sort direction. The selected sector is retained by ID when
values or sorting change. Missing values remain last in either direction.
Click a sector-table header to sort that column; clicking it again reverses the
direction. Click a row to select its sector, or use the mouse wheel to browse.
Mouse and keyboard actions redraw the same cached observation.
Ctrl-C, `q` or Escape cancels cooperatively.

The bordered summary keeps the full integral and accepted result visible above
the sector table, with an iteration/lattice progress bar and process/system RAM.
Scientific result columns align their multiplication dots. The selected-sector
inset shows its point counts, precision dispatch, sample/evaluator times and
measured timing breakdown above the global runtime statistics. Sector timings
describe attributed work; unassigned coordinator work remains in global totals.
Dashboard durations use plain decimal µs, ms and s values.
Sample and operational counts use four significant digits with K, M and B
suffixes (base 1000); small counts remain plain integers.

Havana's mid-batch display is a current-iteration preview. Discrete-sector
marginal errors include the implicit zeros from draws of other sectors; the
total error comes from the total accumulator and preserves selection
correlations. Pilot, production preview and accepted production statistics are
distinct. A resumed preview covers work since resume.

QMC incomplete lattices contribute only progress. One complete shifted lattice
permits a central value; two independent complete shifts permit an uncertainty.
Sector and total coverage can differ. The total uses the selected integration
method's common-shift or independent-sector rule and includes exact offsets.
Preview observations never advance convergence, checkpoints or adaptation.

Scientific notation always includes its exponent. Two significant uncertainty
digits use last-digit parentheses: `0.1234 ± 0.0056` displays as
`+1.234(56)·10⁻¹`. Missing uncertainty is explicitly unavailable, never `(0)`.

The stability histogram classifies each assessed point once after escalation:
f64, DoubleFloat, Arb<1000>, or Unstable. The last category distinguishes explicit
cutoff zeros from failures. Attempted evaluator calls are counted separately.
Historical data lacking a counter is unavailable rather than assumed zero.

Timing separates native evaluator calls from input preparation and escalation
(integrand overhead), and from sampling, accumulation and active scheduling
(integrator overhead). Nested spans are subtracted once; coordinator waiting is
excluded. These percentages describe aggregate instrumented elapsed work across
threads. Actual process CPU time comes separately from operating-system counters.
Dashboard wall time, aggregate worker time and CPU time use the same µs/ms/s
format. Saved values remain raw seconds. Operational measurements include pilot work and
discarded prefixes independently of accepted statistical samples. Average f64
time uses primary f64 evaluator calls only; the slowest sector is identified by
its own mean rather than by a single outlier.

Effective settings, cutoff configuration and accuracy targets enter result
provenance and checkpoint compatibility. Historical checkpoints retain their
original validated policy. Changing a checkpoint's numerical policy on resume
is rejected. See the [ggHH commands](../examples/gghh_double_box/README.md) for
eight-worker Havana and QMC runs with explicit target precision and work limits.
