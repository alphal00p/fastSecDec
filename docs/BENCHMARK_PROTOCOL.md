# Scientific and performance baseline protocol

## Frozen inputs

Reference source: FastSecDecPathFinder
`582d8c7f6dde9bf750750d4c2a2d85a94ce940cd`.
Record the actual installed Symbolica, SymJIT, pySecDec, QMCPy, Python and Rust
versions together with both source revisions and complete command lines.
The external reference environment is prepared. Initial diagnostic runs are
recorded in `reviews/reference-evaluator-smoke.md`; matched repeated baselines
and performance acceptance remain pending. No measured parity is claimed.

Use representative triangle/box, double-box, numerator-heavy multiloop, and hard
four-loop positive-orthant examples. Record exact measure multipliers, parameter
assignments, requested Laurent orders, domains and target coefficients.
Reference sector partitions and numbering are not compatibility requirements.

## Matched configuration

Both sides use SymJIT O2, the same worker count, precision policy, coefficient
range, transforms, and statistical target. For QMC comparisons use complete
sector support and complete Laurent-vector outputs. The reference's QMCPy linear
rank-one backend can use the same Kuo vector, lattice size, and shift count.
Where needed provide identical shifts explicitly for pointwise comparisons.
Record any unavoidable dependency-version differences instead of hiding them.

Record the concrete generating vector, provenance and modulus, not just the
requested point count or a backend label. The frozen reference defaults to a
prime CBC/PT rule and caps requests at 4096 even in democratic mode; a nominal
4096 request uses 4261 points. It also defaults to boundary support, so select
full support explicitly for the matched comparison. See the
[reference rule audit](reviews/reference-qmc-rule-and-refinement.md).
The [native rule-quality investigation](reviews/six-line-qmc-convergence.md)
shows that the current Kuo33002 vector can interact poorly with Korobov3 at
small embedded sizes. Keep diagnostic cross-rule comparisons separate from
fixed-rule timing acceptance; neither an isolated small error bar nor a faster
inaccurate reference path establishes parity.

The installed reference wheel uses Symbolica 2.1.0 with embedded SymJIT 2.18.6;
FastSecDec uses the recorded Symbolica 3.0.1 worktree and SymJIT 2.26.4. Earlier
development diagnostics used SymJIT 2.26.0 and retain that provenance. Initial
box checks show that the reference's real O2 JIT path returns wrong coefficients,
whereas its complex O2 JIT path agrees with its eager evaluator and the analytic
result. Use the independently checked complex path for reference trials until
the real-path defect is resolved. Keep that backend difference explicit in every
report; incorrect real-path timings cannot establish an acceptance baseline.

Do not compare a scalar coefficient kernel with an entire Laurent-vector kernel,
or compare per-sector medians when their decompositions differ. Measure complete
observable work and time to verified accuracy. Record additional historical
backend/rule comparisons separately from the binding matched-O2 gate.

## Measurements

Separate cold symbolic generation/compilation, warm artifact load, warmed batched
kernel evaluation, fixed-work integration, and time to scientific accuracy.
Record peak memory, real evaluation counts, worker-local point generation,
reduction, rescue counts, errors and error-certification state. Cold runs use
empty task-specific caches; warm runs explicitly identify the reused artifacts.

Alternate reference and Rust order. Take seven paired ordinary runs and at least
three expensive generation runs. Save raw timing samples and report medians and
dispersion. A per-case timing ratio above 1.05 fails the agreed gate and requires
investigation/optimization; averaging away a regression is not acceptance.

Use several independent randomization seeds for time-to-accuracy. Compare actual
error against analytic or independently verified coefficients as well as reported
uncertainty. Underestimated uncertainty, discarded failed samples, or a result
without meaningful coverage cannot pass through speed alone.

Linux x86-64 is the currently available execution platform. Record other targets
as unverified until their builds/tests are actually executed.
